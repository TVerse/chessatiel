use crate::lichess::LichessClient;
use bytes::Bytes;
use futures::prelude::stream::*;
use log::*;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GameStateEvent {
    GameFull {
        state: State,
    },
    GameState {
        #[serde(flatten)]
        state: State,
    },
    ChatLine,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub moves: String,
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub status: String,
}

pub struct MakeMove {
    pub chess_move: String,
}

pub struct GameClient {
    lichess_client: LichessClient,
    game_id: String,
}

impl GameClient {
    pub fn new(lichess_client: LichessClient, game_id: String) -> Self {
        Self {
            lichess_client,
            game_id,
        }
    }

    fn game_event_stream_url(&self) -> String {
        return format!(
            "{}/api/bot/game/stream/{}",
            self.lichess_client.lichess_base_url(),
            self.game_id
        );
    }

    fn make_move_url(&self, m: &MakeMove) -> String {
        return format!(
            "{}/api/bot/game/{}/move/{}",
            self.lichess_client.lichess_base_url(),
            self.game_id,
            m.chess_move
        );
    }

    pub async fn get_game_events(
        &self,
    ) -> Result<impl Stream<Item = GameStateEvent>, reqwest::Error> {
        let bytes = self
            .lichess_client
            .client()
            .get(self.game_event_stream_url())
            .bearer_auth(&self.lichess_client.token())
            .send()
            .await?
            .bytes_stream();

        // Assume each chunk is a full response
        let bodies = bytes.filter_map(|b| async { Self::deserialize_ignoring_only_newline(b) });

        Ok(bodies)
    }

    pub async fn submit_move(&self, m: &MakeMove) -> Result<bool, reqwest::Error> {
        self.lichess_client
            .client()
            .post(self.make_move_url(m))
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
    fn deserialize_game_full_event() {
        let json = "{
  \"type\": \"gameFull\",
  \"id\": \"4IrD6Gzz\",
  \"rated\": true,
  \"variant\": {
    \"key\": \"standard\",
    \"name\": \"Standard\",
    \"short\": \"Std\"
  },
  \"clock\": {
    \"initial\": 1199999,
    \"increment\": 9999
  },
  \"speed\": \"classical\",
  \"perf\": {
    \"name\": \"Classical\"
  },
  \"createdAt\": 1523825103561,
  \"white\": {
    \"id\": \"lovlas\",
    \"name\": \"lovlas\",
    \"provisional\": false,
    \"rating\": 2499,
    \"title\": \"IM\"
  },
  \"black\": {
    \"id\": \"leela\",
    \"name\": \"leela\",
    \"rating\": 2389,
    \"title\": null
  },
  \"initialFen\": \"startpos\",
  \"state\": {
    \"type\": \"gameState\",
    \"moves\": \"e1e4 c7c5 f2f4 d7d6 g1f3 b8c6 f1c4 g8f6 d2d3 g7g6 e1g1 f8g7\",
    \"wtime\": 7598039,
    \"btime\": 8395219,
    \"winc\": 9999,
    \"binc\": 9999,
    \"status\": \"started\"
  }
}";

        let result: GameStateEvent = serde_json::from_str(json).unwrap();
        let expected = GameStateEvent::GameFull {
            state: State {
                moves: "e1e4 c7c5 f2f4 d7d6 g1f3 b8c6 f1c4 g8f6 d2d3 g7g6 e1g1 f8g7".to_owned(),
                wtime: 7598039,
                btime: 8395219,
                winc: 9999,
                binc: 9999,
                status: "started".to_owned(),
            },
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn deserialize_game_state_event() {
        let json = "{
  \"type\": \"gameState\",
  \"moves\": \"e1e4 c7c5 f2f4 d7d6 g1f3 b8c6 f1c4 g8f6 d2d3 g7g6 e1g1 f8g7 b1c3\",
  \"wtime\": 7598039,
  \"btime\": 8395219,
  \"winc\": 9999,
  \"binc\": 9999,
  \"status\": \"started\"
}";
        let result: GameStateEvent = serde_json::from_str(json).unwrap();
        let expected = GameStateEvent::GameState {
            state: State {
                moves: "e1e4 c7c5 f2f4 d7d6 g1f3 b8c6 f1c4 g8f6 d2d3 g7g6 e1g1 f8g7 b1c3"
                    .to_owned(),
                wtime: 7598039,
                btime: 8395219,
                winc: 9999,
                binc: 9999,
                status: "started".to_owned(),
            },
        };

        assert_eq!(result, expected)
    }
}
