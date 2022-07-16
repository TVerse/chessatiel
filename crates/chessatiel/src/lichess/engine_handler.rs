use crate::lichess::{GameClient, GameStateEvent};
use futures::prelude::stream::*;
use tokio::sync::oneshot;
use tracing::{debug, error, info_span, Instrument, Span};

use crate::brain::Engine;
use crate::lichess::game::MakeMove;
use anyhow::Result;
use guts::{Move, MoveBuffer, MoveGenerator, Position};
use std::str::FromStr;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug)]
pub struct EngineHandler {
    game_client: GameClient,
    abort: oneshot::Receiver<()>,
    engine: Engine,
}

impl EngineHandler {
    pub fn new(game_client: GameClient, abort: oneshot::Receiver<()>) -> Self {
        Self {
            game_client,
            abort,
            engine: Engine::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let span = info_span!("GameStateEvent");
        self.game_client
            .get_game_events()
            .await?
            .for_each(|r| async {
                match r {
                    Ok(Some(e)) => match e {
                        GameStateEvent::GameFull {
                            immutable_info,
                            state,
                            ..
                        } => {
                            // self.game_client.submit_move(&MakeMove {
                            //     chess_move: m.as_uci()
                            // }).await?;
                        }
                        GameStateEvent::GameState { .. } => {}
                        GameStateEvent::ChatLine => {}
                    }
                    Ok(None) => {
                        debug!("Ignoring keepalive event");
                    }
                    Err(e) => {
                        error!("Got an error in the account stream: {}", e);
                    }
                }
            })
            .await;

        Ok(())
    }
}

struct PositionHistory {
    initial_position: Position,
    moves: Vec<Move>,
}

impl PositionHistory {
    pub fn new(initial_position: Position) -> Self {
        Self {
            initial_position,
            moves: Vec::new(),
        }
    }
}