use crate::lichess::{decode_response, LichessClient};
use anyhow::Result;
use futures::prelude::stream::*;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GameStateEvent {
    GameFull {
        #[serde(flatten)]
        immutable_info: ImmutableInfo,
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
pub struct ImmutableInfo {
    pub clock: Clock,
    pub initial_fen: String,
    pub white: Player,
    pub black: Player,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Player {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub initial: u64,
    pub increment: u64,
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

#[derive(Debug)]
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
    ) -> Result<impl Stream<Item = Result<Option<GameStateEvent>>>> {
        let bytes = self
            .lichess_client
            .client()
            .get(self.game_event_stream_url())
            .send()
            .await?
            .bytes_stream();

        // Assume each chunk is a full response
        Ok(bytes.err_into().and_then(|b| async { decode_response(b) }))
    }

    pub async fn submit_move(&self, m: &MakeMove) -> Result<bool, reqwest::Error> {
        self.lichess_client
            .client()
            .post(self.make_move_url(m))
            .send()
            .await
            .map(|r| r.status() == StatusCode::OK)
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
            immutable_info: ImmutableInfo {
                clock: Clock {
                    initial: 1199999,
                    increment: 9999,
                },
                initial_fen: "startpos".to_owned(),
                white: Player {
                    id: "lovlas".to_string(),
                    name: "lovlas".to_string(),
                },
                black: Player {
                    id: "leela".to_string(),
                    name: "leela".to_string(),
                },
            },
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
