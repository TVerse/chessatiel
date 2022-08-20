use crate::position_hash_history::PositionHashHistory;
use crate::searcher::{Searcher, SearcherConfig};
use crate::time_manager::TimeManagerHandle;
use crate::{answer, AnswerTx, MoveResult, SearchConfiguration};
use guts::Position;
use log::{debug, info};
use tokio::select;
use tokio::sync::watch;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum AggregatorMessage {
    StartSearch(
        AnswerTx<Option<MoveResult>>,
        oneshot::Receiver<()>,
        Position,
        PositionHashHistory,
        SearchConfiguration,
    ),
}

pub struct AggregatorHandle {
    sender: mpsc::UnboundedSender<AggregatorMessage>,
}

impl AggregatorHandle {
    pub fn new(cancellation_rx: watch::Receiver<()>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = AggregatorActor::new(receiver, cancellation_rx, TimeManagerHandle::new());
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start_search(
        &self,
        stop: oneshot::Receiver<()>,
        position: Position,
        position_history: PositionHashHistory,
        search_configuration: SearchConfiguration,
    ) -> Option<MoveResult> {
        let (tx, rx) = answer();
        let msg = AggregatorMessage::StartSearch(
            tx,
            stop,
            position,
            position_history,
            search_configuration,
        );

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct AggregatorActor {
    receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
    cancellation_rx: watch::Receiver<()>,
    time_manager: TimeManagerHandle,
}

impl AggregatorActor {
    async fn handle_event(&mut self, message: AggregatorMessage) {
        debug!("Got aggregator message");
        match message {
            AggregatorMessage::StartSearch(
                answer,
                stop,
                mut position,
                mut position_history,
                config,
            ) => {
                let (result_tx, mut result_rx) = mpsc::unbounded_channel();
                let (stop_tx, stop_rx) = watch::channel(());
                let searcher_stop_rx = stop_rx.clone();
                let _ = self.time_manager.update(config.remaining_time);
                // Should end by itself after cancellation or dropping of the move receiver
                let searcher_config = SearcherConfig {
                    depth: config.depth,
                };
                let _search_task = std::thread::spawn(move || {
                    let mut searcher = Searcher::new(
                        &mut position_history,
                        &mut position,
                        searcher_stop_rx,
                        searcher_config,
                    );
                    searcher.search(result_tx)
                });
                let timer = self.time_manager.start(stop_rx);

                let mut result = None;

                select! {
                    _ = async { while let Some(r) = result_rx.recv().await {
                        result = Some(r)
                    }} => {}
                    _ = timer => {}
                    _ = stop => {
                        let _ = stop_tx.send(());
                    }
                    _ = self.cancellation_rx.changed() => {
                        let _ = stop_tx.send(());
                    }
                }

                info!("Best move found: {:?}", result);

                let _ = answer.send(result);
            }
        }
    }

    fn new(
        receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
        cancellation_rx: watch::Receiver<()>,
        time_manager: TimeManagerHandle,
    ) -> Self {
        Self {
            receiver,
            cancellation_rx,
            time_manager,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }
}
