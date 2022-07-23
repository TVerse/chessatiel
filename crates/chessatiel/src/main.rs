use chessatiel::lichess::{AccountClient, AccountEventHandler, LichessClient};
use clap::Parser;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::select;

use anyhow::Result;
use chessatiel::profiling::{run_profile, ProfileMode};
use futures::prelude::stream::*;
use log::{debug, error};
use log::{info, logger, LevelFilter};
use reqwest::header;
use reqwest::header::{HeaderMap, AUTHORIZATION};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tokio::task::JoinHandle;

const LICHESS_API_TOKEN_PATH: &str = "lichess-api-token";

#[derive(Parser, Debug)]
#[clap()]
struct Args {
    #[clap(short, long, value_enum)]
    profile_mode: Option<ProfileMode>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(profile_mode) = args.profile_mode {
        println!("Entering profile mode {profile_mode:?}");
        let start = Instant::now();
        run_profile(profile_mode);
        let end = Instant::now();
        println!(
            "Running profile command took {:?}",
            end.duration_since(start)
        );
        return Ok(());
    }

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            std::fs::File::create("chessatiel.log").unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();

    periodically_flush_logger(Duration::from_secs(1));

    let token = get_lichess_token().await?;

    let auth_value = format!("Bearer {}", token);

    let mut auth_value = header::HeaderValue::from_str(&auth_value)?;
    auth_value.set_sensitive(true);
    let headers = HeaderMap::from_iter([(AUTHORIZATION, auth_value)]);

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;

    let client = LichessClient::new(client, "https://lichess.org".to_owned());

    let account_client = AccountClient::new(client.clone());
    let event_handler = AccountEventHandler::new(account_client.clone());
    let account_stream = account_client.get_account_stream().await?;

    info!("Ready for events!");
    let handled = account_stream.for_each(|r| async {
        match r {
            Ok(Some(e)) => {
                let _ = event_handler.handle_account_event(e).await;
            }
            Ok(None) => {
                debug!("Ignoring keepalive event");
            }
            Err(e) => {
                error!("Got an error in the account stream: {}", e);
            }
        }
    });

    select! {
        res = handled => { error!("Lichess event stream is not supposed to complete! Result: {:?}", res) }
        _ = tokio::signal::ctrl_c() => {
            info!("Got ctrl-c");
        }
    }

    Ok(())
}

async fn get_lichess_token() -> Result<String> {
    match std::env::var("LICHESS_API_TOKEN") {
        Ok(var) => {
            info!("Got token from env var!");
            return Ok(var)
        },
        Err(_) => {
            info!("Didn't get token from env var, using disk...")
        }
    }
    let mut buf = String::with_capacity(512);
    File::open(LICHESS_API_TOKEN_PATH)
        .await
        .map_err(|_| anyhow::Error::msg("Could not find lichess API token file"))?
        .read_to_string(&mut buf)
        .await?;

    Ok(buf)
}

fn periodically_flush_logger(interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            logger().flush()
        }
    })
}
