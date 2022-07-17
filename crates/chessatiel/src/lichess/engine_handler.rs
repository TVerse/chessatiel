use crate::lichess::{GameClient, GameStateEvent};
use futures::prelude::stream::*;
use log::{debug, error, info};

use crate::brain::{Engine, EngineCommand, MoveResult};
use crate::lichess::game::MakeMove;
use crate::{ack, answer, Shutdown};
use anyhow::Result;
use guts::{Color, Position};
use itertools::Itertools;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const MY_ID: &str = "chessatiel";

#[derive(Debug)]
pub struct EngineHandler {
    game_client: GameClient,
    _shutdown: Shutdown,
    _engine: JoinHandle<bool>,
    engine_commander: mpsc::Sender<EngineCommand>,
}

impl EngineHandler {
    pub(crate) fn new(game_client: GameClient, shutdown: Shutdown) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let engine = Engine::start(shutdown.clone(), rx);
        Self {
            game_client,
            _shutdown: shutdown,
            _engine: engine,
            engine_commander: tx,
        }
    }

    pub async fn run(self) -> Result<()> {
        // TODO handle shutdown

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
                let (tx, rx) = ack();
                self.engine_commander
                    .send(EngineCommand::SetInitialValues(
                        tx,
                        engine_color,
                        Position::from_str(&immutable_info.initial_fen).unwrap_or_else(|_| {
                            panic!(
                                "Lichess sent invalid FEN? Got '{fen}'",
                                fen = immutable_info.initial_fen
                            )
                        }),
                        Self::split_moves(&state.moves),
                    ))
                    .await
                    .unwrap();
                rx.await.unwrap();
                let (tx, rx) = answer();
                self.engine_commander
                    .send(EngineCommand::IsMyMove(tx))
                    .await
                    .unwrap();
                if rx.await.unwrap() {
                    let (tx, rx) = answer();
                    self.engine_commander
                        .send(EngineCommand::Go(tx, true))
                        .await
                        .unwrap();
                    match rx.await.unwrap() {
                        MoveResult::BestMove(m) => {
                            let make_move = MakeMove {
                                chess_move: m.as_uci(),
                            };
                            self.game_client.submit_move(&make_move).await.unwrap();
                        }
                        MoveResult::GameAlreadyFinished => error!("Game was already done?"),
                    }
                }
            }
            GameStateEvent::GameState { state } => {
                let (tx, rx) = ack();
                self.engine_commander
                    .send(EngineCommand::SetMoves(tx, Self::split_moves(&state.moves)))
                    .await
                    .unwrap();
                rx.await.unwrap();
                let (tx, rx) = answer();
                self.engine_commander
                    .send(EngineCommand::IsMyMove(tx))
                    .await
                    .unwrap();
                if rx.await.unwrap() {
                    let (tx, rx) = answer();
                    self.engine_commander
                        .send(EngineCommand::Go(tx, false))
                        .await
                        .unwrap();
                    match rx.await.unwrap() {
                        MoveResult::BestMove(m) => {
                            let make_move = MakeMove {
                                chess_move: m.as_uci(),
                            };
                            self.game_client.submit_move(&make_move).await.unwrap();
                        }
                        MoveResult::GameAlreadyFinished => error!("Game was already done?"),
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
