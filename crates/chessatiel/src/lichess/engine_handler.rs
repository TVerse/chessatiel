use crate::lichess::{GameClient, GameStateEvent};
use log::{debug, error, info, warn};

use crate::lichess::game::{MakeMove, State};
use anyhow::Result;
use brain::{EngineHandle, EngineUpdate, RemainingTime, SearchConfiguration};
use futures::{pin_mut, StreamExt};
use guts::{Color, Position};
use itertools::Itertools;
use std::str::FromStr;
use std::time::Duration;
use tokio::select;
use tokio::sync::watch;
use tokio_stream::wrappers::UnboundedReceiverStream;

const MY_ID: &str = "chessatiel";

pub struct EngineHandler {
    game_client: GameClient,
    engine: EngineHandle,
    my_color: Color,
    cancellation_rx: watch::Receiver<()>,
}

impl EngineHandler {
    pub(crate) fn new(game_client: GameClient, cancellation_rx: watch::Receiver<()>) -> Self {
        let engine = EngineHandle::new(cancellation_rx.clone());
        Self {
            game_client,
            engine,
            cancellation_rx,
            my_color: Color::White,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let stream = self.game_client.get_game_events().await?;
        tokio::pin!(stream);

        while let Some(r) = select! {
            Some(r) = stream.next() => Some(r),
            _ = self.cancellation_rx.changed() => {
                debug!("Engine handler got cancellation notice");
                None
            },
        } {
            match r {
                Ok(Some(e)) => self.handle_game_event(e).await,
                Ok(None) => {
                    debug!("Ignoring keepalive event");
                }
                Err(e) => {
                    error!("Got an error in the account stream: {}", e);
                }
            }
        }

        debug!("Done handling game, shutting down engine handler");

        Ok(())
    }

    async fn handle_game_event(&mut self, e: GameStateEvent) {
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
                self.my_color = engine_color;
                self.engine
                    .set_initial_values(
                        Position::from_str(&immutable_info.initial_fen).unwrap_or_else(|_| {
                            panic!(
                                "Lichess sent invalid FEN? Got '{fen}'",
                                fen = immutable_info.initial_fen
                            )
                        }),
                        Self::split_moves(&state.moves),
                    )
                    .await;
                if self.is_my_move().await {
                    let stream = UnboundedReceiverStream::new(
                        self.engine
                            .go(self.build_configuration(true, &state))
                            .await
                            .unwrap(),
                    )
                    .filter_map(|update| async {
                        match update {
                            EngineUpdate::BestMove(m) => Some(m),
                            _ => None,
                        }
                    });
                    pin_mut!(stream);
                    if let Some(chess_move) = stream
                        .next()
                        .await
                        .and_then(|mr| mr.first_move().cloned())
                        .map(|m| m.as_uci())
                    {
                        let make_move = MakeMove { chess_move };
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
                let stream = UnboundedReceiverStream::new(
                    self.engine
                        .go(self.build_configuration(false, &state))
                        .await
                        .unwrap(),
                )
                .filter_map(|update| async {
                    match update {
                        EngineUpdate::BestMove(m) => Some(m),
                        update => {
                            info!("Got an engine update: {update:?}");
                            None
                        }
                    }
                });
                pin_mut!(stream);
                if let Some(chess_move) = stream
                    .next()
                    .await
                    .and_then(|mr| mr.first_move().cloned())
                    .map(|m| m.as_uci())
                {
                    let make_move = MakeMove { chess_move };
                    if !self.game_client.submit_move(&make_move).await.unwrap() {
                        error!("Got a non-200 from Lichess when making a move");
                        self.game_client.resign().await.unwrap();
                    };
                }
            }
            GameStateEvent::ChatLine => {
                info!("Got a chat message!")
            }
        }
    }

    async fn is_my_move(&self) -> bool {
        self.my_color == self.engine.current_color().await
    }

    fn build_configuration(&self, is_first_move: bool, state: &State) -> SearchConfiguration {
        let remaining_time = if is_first_move {
            Some(RemainingTime::ForMove(Duration::from_secs(15)))
        } else {
            let time = state.time_for(self.my_color);
            Some(RemainingTime::ForGame {
                remaining: time.time,
                increment: time.increment,
            })
        };
        SearchConfiguration {
            remaining_time,
            ..SearchConfiguration::default()
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
