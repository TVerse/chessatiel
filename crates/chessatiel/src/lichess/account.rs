use crate::lichess::decode_response;
use crate::lichess::engine_handler::EngineHandler;
use crate::lichess::{GameClient, LichessClient};
use crate::Shutdown;
use anyhow::Result;
use futures::prelude::stream::*;
use log::{debug, error, info};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum LichessEvent {
    GameStart { game: Game },
    // TODO
    Challenge { challenge: Challenge },
    // TODO
    ChallengeCanceled,
    // TODO
    GameFinish { game: Game },
    // Should never happen, we don't send challenges
    // TODO
    ChallengeDeclined,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: String,
    pub challenger: ChallengeUser,
    pub variant: Variant,
    pub rated: bool,
    pub speed: String,
    // TODO enum
    pub time_control: TimeControl,
    pub color: String, // TODO enum
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeUser {
    pub id: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub key: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum TimeControl {
    Unlimited,
    Clock { limit: u64, increment: u64 },
}

#[derive(Debug)]
struct GameHandle {
    join_handle: JoinHandle<Result<()>>,
    abort_channel: watch::Sender<()>,
}

#[derive(Debug)]
pub struct AccountEventHandler {
    in_progress_games: Mutex<HashMap<String, GameHandle>>,
    client: AccountClient,
}

impl AccountEventHandler {
    pub fn new(client: AccountClient) -> Self {
        Self {
            in_progress_games: Mutex::new(HashMap::with_capacity(10)),
            client,
        }
    }

    pub async fn handle_account_event(
        &self,
        event: LichessEvent,
        client: &LichessClient,
    ) -> Result<()> {
        debug!("Handling account event {:?}", event);
        match event {
            LichessEvent::Challenge { challenge } => {
                if self.should_accept_challenge(&challenge).await {
                    info!("Accepting challenge {:?}", challenge);
                    self.client.accept_challenge(&challenge.id).await?;
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
                // TODO don't await here, run concurrently
                let game_client = GameClient::new(client.clone(), game.id.clone());
                let (tx, rx) = watch::channel(());
                let engine_handler = EngineHandler::new(game_client, Shutdown::new(rx));
                let join_handle = tokio::spawn(engine_handler.run());
                let game_handle = GameHandle {
                    join_handle,
                    abort_channel: tx,
                };
                self.in_progress_games
                    .lock()
                    .await
                    .insert(game.id, game_handle);
                Ok(())
            }
            LichessEvent::GameFinish { game } => {
                info!("Game finish");
                match self.in_progress_games.lock().await.remove(&game.id) {
                    Some(handle) => {
                        match handle.abort_channel.send(()) {
                            Ok(_) => (),
                            Err(_) => {
                                debug!("Game {} finished before we could send the message", game.id)
                            }
                        };
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        handle.join_handle.abort();
                        match handle.join_handle.await {
                            Ok(_) => {}
                            Err(e) => debug!("Got error waiting to join engine: {}", e),
                        };
                    }
                    None => error!("Wanted to remove game {} but not found in map!", game.id),
                };
                Ok(())
            }
            LichessEvent::ChallengeDeclined => todo!(),
        }
    }

    async fn should_accept_challenge(&self, challenge: &Challenge) -> bool {
        // Race condition with accepting game
        self.in_progress_games.lock().await.len() < 1
            && challenge.challenger.id == "dragnmn"
            && challenge.variant.key == "standard"
            && !challenge.rated
            && challenge.time_control != TimeControl::Unlimited
    }
}

#[derive(Debug, Clone)]
pub struct AccountClient {
    base_client: LichessClient,
}

impl AccountClient {
    pub fn new(base_client: LichessClient) -> Self {
        Self { base_client }
    }

    fn lichess_event_stream_url(&self) -> String {
        return format!("{}/api/stream/event", self.base_client.lichess_base_url());
    }

    fn challenge_base_url(&self, challenge_id: &str) -> String {
        return format!(
            "{}/api/challenge/{}",
            self.base_client.lichess_base_url(),
            challenge_id
        );
    }

    fn challenge_accept_url(&self, challenge_id: &str) -> String {
        return format!("{}/accept", self.challenge_base_url(challenge_id));
    }

    pub async fn get_account_stream(
        &self,
    ) -> Result<impl Stream<Item = Result<Option<LichessEvent>>>> {
        let bytes = self
            .base_client
            .client()
            .get(self.lichess_event_stream_url())
            .send()
            .await?
            .bytes_stream();

        // Assume each chunk is a full response
        Ok(bytes.err_into().and_then(|b| async { decode_response(b) }))
    }

    pub async fn accept_challenge(&self, challenge_id: &str) -> Result<bool> {
        self.base_client
            .client()
            .post(self.challenge_accept_url(challenge_id))
            .send()
            .await
            .map(|r| r.status() == StatusCode::OK)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_challenge_event() {
        let json = "{
            \"type\": \"challenge\",
  \"challenge\": {
    \"id\": \"bGeawCe6\",
    \"url\": \"https://lichess.org/bGeawCe6\",
    \"status\": \"created\",
    \"challenger\": {
      \"id\": \"dragnmn\",
      \"name\": \"Dragnmn\",
      \"title\": null,
      \"rating\": 1499,
      \"provisional\": true,
      \"online\": true
    },
    \"destUser\": {
      \"id\": \"chessatiel\",
      \"name\": \"Chessatiel\",
      \"title\": \"BOT\",
      \"rating\": 1499,
      \"provisional\": true,
      \"online\": true
    },
    \"variant\": {
      \"key\": \"standard\",
      \"name\": \"Standard\",
      \"short\": \"Std\"
    },
    \"rated\": false,
    \"speed\": \"correspondence\",
    \"timeControl\": {
      \"type\": \"unlimited\"
    },
    \"color\": \"random\",
    \"perf\": {
      \"icon\": \"u{e01e}\",
      \"name\": \"Correspondence\"
    }
  },
  \"compat\": {
    \"bot\": true,
    \"board\": true
  }
}";
        let result: LichessEvent = serde_json::from_str(json).unwrap();
        let expected = LichessEvent::Challenge {
            challenge: Challenge {
                id: "bGeawCe6".to_owned(),
                challenger: ChallengeUser {
                    id: "dragnmn".to_owned(),
                },
                variant: Variant {
                    key: "standard".to_owned(),
                },
                rated: false,
                speed: "correspondence".to_owned(),
                time_control: TimeControl::Unlimited,
                color: "random".to_owned(),
            },
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_time_control() {
        let json_unlimited = "{
    \"type\": \"unlimited\"
}";
        let result: TimeControl = serde_json::from_str(json_unlimited).unwrap();
        let expected = TimeControl::Unlimited;
        assert_eq!(result, expected);

        let json_standard = "{
    \"type\": \"clock\",
    \"limit\": 299,
    \"increment\": 7,
    \"show\": \"4+8\"
}";
        let result: TimeControl = serde_json::from_str(json_standard).unwrap();
        let expected = TimeControl::Clock {
            limit: 299,
            increment: 7,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_game_start_event() {
        let json = "{
    \"type\": \"gameStart\",
    \"game\": {
        \"id\": \"0lsvP62l\",
        \"compat\": {}
    }
}";
        let result: LichessEvent = serde_json::from_str(json).unwrap();
        let expected = LichessEvent::GameStart {
            game: Game {
                id: "0lsvP62l".to_owned(),
            },
        };

        assert_eq!(result, expected);
    }
}
