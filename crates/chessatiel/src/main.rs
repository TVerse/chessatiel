use log::*;

use anyhow::Result;
use chessatiel::lichess::{AccountEventHandler, LichessClient};
use reqwest::Client;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::Duration;

const LICHESS_API_TOKEN_PATH: &str = "lichess-api-token";

#[tokio::main]
async fn main() -> Result<()> {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            std::fs::File::create("chessatiel.log").unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();
    periodically_flush_logger(Duration::from_secs(1));

    info!("Initializing...");

    let client = Client::new();

    let token = get_lichess_token().await?;

    let lichess_client = LichessClient::new(client, token, "https://lichess.org".to_owned());

    let event_handler = AccountEventHandler::new(lichess_client.clone());

    let event_handler = event_handler.handle_events();

    select! {
        res = event_handler => { panic!("Lichess event stream is not supposed to complete! Result: {:?}", res) }
        _ = tokio::signal::ctrl_c() => {
            info!("Got ctrl-c");
            log::logger().flush()
        }
    }

    Ok(())
}

fn periodically_flush_logger(interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            logger().flush()
        }
    })
}

async fn get_lichess_token() -> Result<String> {
    let mut buf = String::with_capacity(512);
    File::open(LICHESS_API_TOKEN_PATH)
        .await?
        .read_to_string(&mut buf)
        .await?;

    Ok(buf)
}
