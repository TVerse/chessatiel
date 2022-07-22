use crate::lichess::{GameClient, GameStateEvent};
use futures::prelude::stream::*;
use log::{debug, error, info, warn};

use crate::brain::EngineHandle;
use crate::lichess::game::MakeMove;
use anyhow::Result;
use guts::{Color, Position};
use itertools::Itertools;
use std::str::FromStr;

const MY_ID: &str = "chessatiel";

pub struct EngineHandler {
    game_client: GameClient,
    engine: EngineHandle,
}

impl EngineHandler {
    pub(crate) fn new(game_client: GameClient) -> Self {
        let engine = EngineHandle::new();
        Self {
            game_client,
            engine,
        }
    }

    pub async fn run(self) -> Result<()> {
        self.handle_events().await?;

        Ok(())
    }

    async fn handle_events(&self) -> Result<()> {
        self.game_client
            .get_game_events()
            .await?
            .for_each(|r| async {
                match r {
                    Ok(Some(e)) => self.handle_game_event(e).await,
                    Ok(None) => {
                        debug!("Ignoring keepalive event");
                    }
                    Err(e) => {
                        error!("Got an error in the account stream: {}", e);
                    }
                }
            })
            .await;

        debug!("Done handling game, shutting down engine handler");

        Ok(())
    }

    async fn handle_game_event(&self, e: GameStateEvent) {
        match e {
            GameStateEvent::GameFull {
                immutable_info,
                state,
                ..
            } => {
                // TODO this must handle an event fully before parsing the next
                let engine_color = if immutable_info.white.id == MY_ID {
                    Color::White
                } else {
                    Color::Black
                };
                self.engine
                    .set_initial_values(
                        engine_color,
                        Position::from_str(&immutable_info.initial_fen).unwrap_or_else(|_| {
                            panic!(
                                "Lichess sent invalid FEN? Got '{fen}'",
                                fen = immutable_info.initial_fen
                            )
                        }),
                        Self::split_moves(&state.moves),
                    )
                    .await;
                if self.engine.is_my_move().await {
                    if let Some(result) = self.engine.go(true).await {
                        let make_move = MakeMove {
                            chess_move: result.first_move().as_uci(),
                        };
                        self.game_client.submit_move(&make_move).await.unwrap();
                    }
                }
            }
            GameStateEvent::GameState { state } => {
                if state.status != "started" {
                    warn!(
                        "Got a message for a not-running game, aborting, got state: {}",
                        state.status
                    );
                    return;
                };
                self.engine.set_moves(Self::split_moves(&state.moves)).await;
                if self.engine.is_my_move().await {
                    if let Some(result) = self.engine.go(false).await {
                        let make_move = MakeMove {
                            chess_move: result.first_move().as_uci(),
                        };
                        if !self.game_client.submit_move(&make_move).await.unwrap() {
                            error!("Got a non-200 from Lichess when making a move");
                            self.game_client.resign().await.unwrap();
                        };
                    }
                }
            }
            GameStateEvent::ChatLine => {
                info!("Got a chat message!")
            }
        }
    }

    fn split_moves(moves: &str) -> Vec<String> {
        moves
            .split(' ')
            .filter(|m| !m.is_empty())
            .map(|s| s.to_owned())
            .collect_vec()
    }
}
