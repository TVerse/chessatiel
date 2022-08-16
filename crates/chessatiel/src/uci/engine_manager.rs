use crate::uci::protocol::{IncomingCommand, InfoPayload, OutgoingCommand};
use brain::EngineHandle;
use guts::Color;
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
                    let (cancellation_tx, cancellation_rx) = watch::channel(());
                    let engine_handle = EngineHandle::new(cancellation_rx);
                    self.cancellation_tx = cancellation_tx;
                    self.engine_handle = engine_handle;
                }
                IncomingCommand::Position(pos, moves) => {
                    // Color does not matter here
                    self.engine_handle
                        .set_initial_values(Color::White, pos, moves)
                        .await
                }
                IncomingCommand::Go(_) => {
                    let result = self.engine_handle.go(false).await;
                    if let Some(m) = result.and_then(|r| r.first_move().cloned()) {
                        self.tx.send(OutgoingCommand::BestMove(m.as_uci())).unwrap()
                    } else {
                        self.tx
                            .send(OutgoingCommand::Info(InfoPayload::String(
                                "Did not find best move!".to_string(),
                            )))
                            .unwrap()
                    }
                }
                IncomingCommand::Stop => {
                    self.cancellation_tx.send(()).unwrap();
                    // TODO grab current best move
                }
                IncomingCommand::Quit => break,
            }
        }
    }
}
