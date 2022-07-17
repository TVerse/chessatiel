use std::fmt::Debug;

mod account;
mod engine_handler;
mod game;

use anyhow::Result;
use bytes::Bytes;
use reqwest::Client;
use serde::de::DeserializeOwned;
use tracing::instrument;
use tracing::{debug, error, info};

pub use crate::lichess::account::{
    AccountClient, AccountEventHandler, Challenge, LichessEvent, TimeControl,
};

pub use crate::lichess::game::{GameClient, GameStateEvent};

#[derive(Debug, Clone)]
pub struct LichessClient {
    client: Client,
    lichess_base_url: String,
}

impl LichessClient {
    pub fn new(client: Client, lichess_base_url: String) -> Self {
        Self {
            client,
            lichess_base_url,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn lichess_base_url(&self) -> &str {
        &self.lichess_base_url
    }
}

#[instrument]
fn decode_response<T: DeserializeOwned + Debug>(bytes: Bytes) -> Result<Option<T>> {
    match String::from_utf8(bytes.to_vec()) {
        Ok(str) if str != "\n" => {
            debug!("Possibly got an event: {:?}", str);
            match serde_json::from_slice(&bytes) {
                Ok(event) => {
                    info!("Got an account event: {:?}", event);
                    Ok(event)
                }
                Err(e) => {
                    error!("Error decoding json. Error: {}, source: {:?}", e, str);
                    Err(e.into())
                }
            }
        }
        Ok(str) => {
            debug!("Ignoring keepalive string {:?}", str);
            Ok(None)
        }
        Err(e) => {
            error!("Error decoding bytes from stream: {}", e);
            Err(e.into())
        }
    }
}
