use log::debug;
use log::error;
use log::info;
use std::collections::HashMap;

use futures::prelude::stream::*;
mod account;
pub(crate) mod game;

use anyhow::Result;
use reqwest::Client;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

use crate::engine::Engine;
use crate::lichess::account::{AccountClient, Challenge, LichessEvent, TimeControl};

pub struct AccountEventHandler {
    account_client: AccountClient,
    lichess_client: LichessClient,
    in_progress_games: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl AccountEventHandler {
    pub fn new(lichess_client: LichessClient) -> Self {
        Self {
            account_client: AccountClient::new(lichess_client.clone()),
            lichess_client,
            in_progress_games: Mutex::new(HashMap::with_capacity(10)),
        }
    }

    pub async fn handle_events(&self) -> Result<()> {
        self.account_client
            .get_lichess_events()
            .await?
            .for_each(|e| async {
                debug!("Got event: {:?}", e);
                match self.handle_event(e).await {
                    Ok(()) => (),
                    Err(e) => error!("Got an error handling account event: {}", e),
                }
            })
            .await;

        Ok(())
    }

    async fn handle_event(&self, event: LichessEvent) -> Result<()> {
        debug!("Handling account event {:?}", event);
        match event {
            LichessEvent::Challenge { challenge } => {
                if self.should_accept_challenge(&challenge).await {
                    info!("Accepting challenge {:?}", challenge);
                    self.account_client.accept_challenge(&challenge.id).await?;
                    Ok(())
                } else {
                    info!("Would not accept challenge {:?}", challenge);
                    Ok(())
                }
            }
            LichessEvent::ChallengeCanceled => {
                info!("Challenge canceled");
                Ok(())
            }
            LichessEvent::GameStart { game } => {
                info!("Game started: {}", game.id);
                let engine = Engine::new(self.lichess_client.clone(), game.id.clone());
                let tx = engine.run().await;
                self.in_progress_games.lock().await.insert(game.id, tx);
                Ok(())
            }
            LichessEvent::GameFinish { game } => {
                info!("Game finish");
                match self.in_progress_games.lock().await.remove(&game.id) {
                    Some(tx) => match tx.send(()) {
                        Ok(_) => (),
                        Err(_) => {
                            debug!("Game {} finished before we could send the message", game.id)
                        }
                    },
                    None => error!("Wanted to remove game {} but not found in map!", game.id),
                };
                Ok(())
            }
            LichessEvent::ChallengeDeclined => todo!(),
        }
    }

    async fn should_accept_challenge(&self, challenge: &Challenge) -> bool {
        self.in_progress_games.lock().await.len() < 1
            && challenge.challenger.id == "dragnmn"
            && challenge.variant.key == "standard"
            && !challenge.rated
            && challenge.time_control != TimeControl::Unlimited
    }
}

#[derive(Clone)]
pub struct LichessClient {
    client: Client,
    token: String,
    lichess_base_url: String,
}

impl LichessClient {
    pub fn new(client: Client, token: String, lichess_base_url: String) -> Self {
        Self {
            client,
            token,
            lichess_base_url,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn lichess_base_url(&self) -> &str {
        &self.lichess_base_url
    }
}
