use chessatiel::lichess::{AccountClient, AccountEventHandler, LichessClient};
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::select;

use anyhow::Result;
use futures::prelude::stream::*;
use reqwest::header;
use reqwest::header::{HeaderMap, AUTHORIZATION};
use tracing::info;
use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

const LICHESS_API_TOKEN_PATH: &str = "lichess-api-token";

#[derive(StructOpt, Debug)]
#[structopt(name = "Chessatiel")]
struct Opt {}

#[tokio::main]
async fn main() -> Result<()> {
    let tracer =
        opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(telemetry)
        .init();

    let _opt = Opt::from_args();

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

    let handled = account_stream.for_each(|r| async {
        match r {
            Ok(Some(e)) => {
                let _ = event_handler.handle_event(e, &client).await;
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

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn get_lichess_token() -> Result<String> {
    let mut buf = String::with_capacity(512);
    File::open(LICHESS_API_TOKEN_PATH)
        .await?
        .read_to_string(&mut buf)
        .await?;

    Ok(buf)
}
