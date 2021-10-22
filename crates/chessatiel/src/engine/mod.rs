mod position_manager;

use crate::engine::position_manager::{PositionManager, PositionManagerCommand};
use crate::lichess::game::{GameClient, GameStateEvent};
use crate::lichess::LichessClient;
use anyhow::Result;
use futures::prelude::stream::*;
use guts::{Move, MoveGenerator, Position};
use log::*;
use std::str::FromStr;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub fn build_engine_net(lichess_client: LichessClient, game_id: String) -> Engine {
    let move_generator = MoveGenerator::new();
    let (pmc_tx, pmc_rx) = mpsc::channel(10);
    let (pos_tx, pos_rx) = mpsc::channel(10);
    let position_manager = PositionManager::new(move_generator, pmc_rx, pos_tx);
    position_manager.run();
    Engine::new(lichess_client, game_id, pmc_tx)
}

pub struct Engine {
    game_client: GameClient,
    game_id: String,
    best_move: Option<Move>,
    position_manager_tx: mpsc::Sender<PositionManagerCommand>,
}

impl Engine {
    pub fn new(
        lichess_client: LichessClient,
        game_id: String,
        position_manager_tx: mpsc::Sender<PositionManagerCommand>,
    ) -> Self {
        Self {
            game_client: GameClient::new(lichess_client, game_id.clone()),
            game_id,
            best_move: None,
            position_manager_tx,
        }
    }

    pub async fn run(self) -> oneshot::Sender<()> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            // TODO game end also closes the connection, is this select necessary?
            let handle_events = self.handle_events();
            select! {
                _res = handle_events => { info!("Done handling game {} (by handler)", self.game_id) }
                _res = rx => { info!("Done handling game {} (by channel)", self.game_id) }
            }
        });
        tx
    }

    pub async fn handle_events(&self) -> Result<()> {
        self.game_client
            .get_game_events()
            .await?
            .for_each(|e| async {
                match self.handle_event(e).await {
                    Ok(()) => (),
                    Err(e) => error!("Got an error handling game event: {}", e),
                }
            })
            .await;
        Ok(())
    }

    async fn handle_event(&self, event: GameStateEvent) -> Result<()> {
        debug!("Handling game event {:?}", event);
        match event {
            GameStateEvent::GameFull {
                immutable_info,
                state,
                ..
            } => {
                let starting_position =
                    Position::from_str(&immutable_info.initial_fen).expect(&format!(
                        "Lichess sent invalid FEN? Got: {:?}",
                        immutable_info.initial_fen
                    ));
                let moves = state.moves.split(" ").map(|s| s.to_owned()).collect();
                self.position_manager_tx
                    .send(PositionManagerCommand::SetPosition(
                        starting_position,
                        moves,
                    ))
                    .await?;
                Ok(())
            }
            GameStateEvent::GameState { .. } => Ok(()),
            GameStateEvent::ChatLine => {
                info!("Got a chat message");
                Ok(())
            }
        }
    }
}
