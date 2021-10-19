use bytes::Bytes;
use futures::prelude::stream::*;
use log::debug;
use log::error;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::lichess::LichessClient;
use serde::Deserialize;

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
    pub speed: String, // TODO enum
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

    pub async fn get_lichess_events(
        &self,
    ) -> Result<impl Stream<Item = LichessEvent>, reqwest::Error> {
        let bytes = self
            .base_client
            .client()
            .get(self.lichess_event_stream_url())
            .bearer_auth(&self.base_client.token())
            .send()
            .await?
            .bytes_stream();

        // Assume each chunk is a full response
        let bodies = bytes.filter_map(|b| async { Self::deserialize_ignoring_only_newline(b) });

        Ok(bodies)
    }

    pub async fn accept_challenge(&self, challenge_id: &str) -> Result<bool, reqwest::Error> {
        self.base_client
            .client()
            .post(self.challenge_accept_url(challenge_id))
            .bearer_auth(&self.base_client.token())
            .send()
            .await
            .map(|r| r.status() == StatusCode::OK)
    }

    fn deserialize_ignoring_only_newline<T: DeserializeOwned>(
        b: Result<Bytes, reqwest::Error>,
    ) -> Option<T> {
        match b {
            Ok(b) => match String::from_utf8(b.to_vec()) {
                Ok(str) if str != "\n" => {
                    debug!("Deserializing raw request: {:?}", str);
                    match serde_json::from_slice(&b) {
                        Ok(e) => Some(e),
                        Err(e) => {
                            error!("Error decoding json. Error: {}, source: {:?}", e, str);
                            None
                        }
                    }
                }
                Ok(str) => {
                    debug!("Ignoring keepalive string {:?}", str);
                    None
                }
                Err(e) => {
                    error!("Did not get valid utf-8 from event stream: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Error retrieving bytes from stream: {}", e);
                None
            }
        }
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
