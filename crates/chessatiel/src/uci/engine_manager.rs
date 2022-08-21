use crate::uci::protocol::{GoPayload, IncomingCommand, InfoPayload, OutgoingCommand};
use brain::{EngineHandle, RemainingTime, SearchConfiguration};
use guts::Color;
use log::debug;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch;

pub struct EngineManager {
    rx: UnboundedReceiver<IncomingCommand>,
    tx: UnboundedSender<OutgoingCommand>,
    cancellation_tx: watch::Sender<()>,
    engine_handle: EngineHandle,
}

impl EngineManager {
    pub fn new(
        rx: UnboundedReceiver<IncomingCommand>,
        tx: UnboundedSender<OutgoingCommand>,
    ) -> Self {
        let (cancellation_tx, cancellation_rx) = watch::channel(());
        let engine_handle = EngineHandle::new(cancellation_rx);
        Self {
            rx,
            tx,
            cancellation_tx,
            engine_handle,
        }
    }

    pub async fn run(mut self) {
        while let Some(incoming_command) = self.rx.recv().await {
            debug!("Got command: {incoming_command}");
            match incoming_command {
                IncomingCommand::Uci => {
                    self.tx
                        .send(OutgoingCommand::Id("name", "Chessatiel"))
                        .unwrap();
                    self.tx
                        .send(OutgoingCommand::Id(
                            "author",
                            "Tim E (https://lichess.org/@/Dragnmn)",
                        ))
                        .unwrap();
                    self.tx.send(OutgoingCommand::UciOk).unwrap();
                }
                IncomingCommand::Debug(_) => {}
                IncomingCommand::IsReady => {
                    self.tx.send(OutgoingCommand::ReadyOk).unwrap();
                }
                IncomingCommand::UciNewGame => {
                    let _ = self.cancellation_tx.send(());
                    let (cancellation_tx, cancellation_rx) = watch::channel(());
                    let engine_handle = EngineHandle::new(cancellation_rx);
                    self.cancellation_tx = cancellation_tx;
                    self.engine_handle = engine_handle;
                }
                IncomingCommand::Position(pos, moves) => {
                    self.engine_handle.set_initial_values(pos, moves).await
                }
                IncomingCommand::Go(go_payload) => {
                    let tx = self.tx.clone();
                    let current_color = self.engine_handle.current_color().await;
                    match self
                        .engine_handle
                        .go(self.build_configuration(go_payload, current_color))
                        .await
                    {
                        Ok(maybe_result_rx) => {
                            tokio::task::spawn(async move {
                                if let Some(m) = maybe_result_rx
                                    .await
                                    .unwrap()
                                    .and_then(|r| r.first_move().cloned())
                                {
                                    tx.send(OutgoingCommand::BestMove(m.as_uci())).unwrap()
                                } else {
                                    tx.send(OutgoingCommand::Info(InfoPayload::String(
                                        "Did not find best move!".to_string(),
                                    )))
                                    .unwrap()
                                }
                            });
                        }
                        Err(e) => {
                            let _ =
                                self.tx
                                    .send(OutgoingCommand::Info(InfoPayload::String(format!(
                                        "Tried to start a search, but had an error: {e}"
                                    ))));
                        }
                    };
                }
                IncomingCommand::Stop => {
                    let _ = self.engine_handle.stop().await;
                }
                IncomingCommand::Quit => break,
            }
        }
    }

    fn build_configuration(&self, go_payload: GoPayload, color: Color) -> SearchConfiguration {
        let remaining_time = if let Some(movetime) = go_payload.move_time {
            Some(RemainingTime::ForMove(movetime))
        } else {
            match color {
                Color::White => go_payload.wtime.map(|d| RemainingTime::ForGame {
                    remaining: d,
                    increment: go_payload.winc.unwrap_or(Duration::from_secs(0)),
                }),
                Color::Black => go_payload.btime.map(|d| RemainingTime::ForGame {
                    remaining: d,
                    increment: go_payload.binc.unwrap_or(Duration::from_secs(0)),
                }),
            }
        };

        SearchConfiguration {
            depth: go_payload.depth,
            remaining_time,
        }
    }
}
