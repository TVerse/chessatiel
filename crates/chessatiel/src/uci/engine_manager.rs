use crate::uci::protocol::{GoPayload, IncomingCommand, InfoPayload, OutgoingCommand};
use brain::{EngineHandle, RemainingTime, SearchConfiguration};
use log::debug;
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
                            "Tim E (https://lichess.org/@/Dragnmn",
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
                    match self
                        .engine_handle
                        .go(self.build_configuration(go_payload))
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

    fn build_configuration(&self, go_payload: GoPayload) -> SearchConfiguration {
        match go_payload {
            GoPayload::Perft(_) => todo!(),
            GoPayload::Depth(d) => SearchConfiguration {
                depth: Some(d),
                ..SearchConfiguration::default()
            },
            GoPayload::Movetime(t) => SearchConfiguration {
                remaining_time: Some(RemainingTime::ForMove(t)),
                ..SearchConfiguration::default()
            },
        }
    }
}
