use crate::lichess::decode_response;
use crate::lichess::engine_handler::EngineHandler;
use crate::lichess::{GameClient, LichessClient};
use anyhow::Result;
use futures::prelude::stream::*;
use log::{debug, error, info};
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::{watch, Mutex};

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

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DeclineChallenge {
    reason: DeclineReason,
}

#[derive(Serialize, Debug, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DeclineReason {
    Generic,
    Later,
    TooFast,
    TooSlow,
    TimeControl,
    Rated,
    Casual,
    Standard,
    Variant,
    NoBot,
    OnlyBot,
}

#[derive(Debug)]
struct GameHandle {
    cancellation_tx: watch::Sender<()>,
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

    pub async fn handle_account_event(&self, event: LichessEvent) -> Result<()> {
        debug!("Handling account event {:?}", event);
        match event {
            LichessEvent::Challenge { challenge } => {
                if let Some(decline_reason) = self.should_accept_challenge(&challenge).await {
                    info!(
                        "Did not accept challenge {:?} for reason {:?}",
                        challenge, decline_reason
                    );
                    self.client
                        .decline_challenge(&challenge.id, decline_reason)
                        .await?;
                    Ok(())
                } else {
                    info!("Accepting challenge {:?}", challenge);
                    self.client.accept_challenge(&challenge.id).await?;
                    Ok(())
                }
            }
            LichessEvent::ChallengeCanceled => {
                info!("Challenge canceled");
                Ok(())
            }
            LichessEvent::GameStart { game } => {
                info!("Game started: {}", game.id);
                let game_client = GameClient::new(self.client.base_client.clone(), game.id.clone());
                let (cancellation_tx, cancellation_rx) = watch::channel(());
                let mut engine_handler = EngineHandler::new(game_client, cancellation_rx);
                let _ = tokio::spawn(async move { engine_handler.run().await });
                let game_handle = GameHandle { cancellation_tx };
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
                        debug!("Removed game {} from in progress games", game.id);
                        handle.cancellation_tx.send(()).unwrap();
                    }
                    None => error!("Wanted to remove game {} but not found in map!", game.id),
                };
                Ok(())
            }
            LichessEvent::ChallengeDeclined => Ok(()),
        }
    }

    async fn should_accept_challenge(&self, challenge: &Challenge) -> Option<DeclineReason> {
        if self.in_progress_games.lock().await.len() >= 1 {
            info!("Too many in-progress games");
            Some(DeclineReason::Generic)
        } else if challenge.challenger.id != "dragnmn" {
            info!("Got challenge by wrong account");
            Some(DeclineReason::Generic)
        } else if challenge.variant.key != "standard" {
            Some(DeclineReason::Variant)
        } else if challenge.rated {
            Some(DeclineReason::Casual)
        } else if challenge.time_control == TimeControl::Unlimited {
            Some(DeclineReason::TimeControl)
        } else {
            None
        }
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
        format!("{}/api/stream/event", self.base_client.lichess_base_url())
    }

    fn challenge_base_url(&self, challenge_id: &str) -> String {
        format!(
            "{}/api/challenge/{}",
            self.base_client.lichess_base_url(),
            challenge_id
        )
    }

    fn challenge_accept_url(&self, challenge_id: &str) -> String {
        format!("{}/accept", self.challenge_base_url(challenge_id))
    }

    fn challenge_decline_url(&self, challenge_id: &str) -> String {
        format!("{}/decline", self.challenge_base_url(challenge_id))
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

    pub async fn decline_challenge(
        &self,
        challenge_id: &str,
        decline_reason: DeclineReason,
    ) -> Result<bool> {
        self.base_client
            .client()
            .post(self.challenge_decline_url(challenge_id))
            .json(&DeclineChallenge {
                reason: decline_reason,
            })
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
