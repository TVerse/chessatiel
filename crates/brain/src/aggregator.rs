use crate::position_hash_history::PositionHashHistory;
use crate::searcher::Searcher;
use crate::{answer, AnswerTx, MoveResult};
use guts::Position;
use log::{debug, info};
use tokio::sync::mpsc;
use tokio::sync::watch;

#[derive(Debug)]
enum AggregatorMessage {
    StartSearch(AnswerTx<Option<MoveResult>>, Position, PositionHashHistory),
}

pub struct AggregatorHandle {
    sender: mpsc::UnboundedSender<AggregatorMessage>,
}

impl AggregatorHandle {
    pub fn new(cancellation_rx: watch::Receiver<()>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = AggregatorActor::new(receiver, cancellation_rx);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start_search(
        &self,
        position: Position,
        position_history: PositionHashHistory,
    ) -> Option<MoveResult> {
        let (tx, rx) = answer();
        let msg = AggregatorMessage::StartSearch(tx, position, position_history);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct AggregatorActor {
    receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
    cancellation_rx: watch::Receiver<()>,
}

impl AggregatorActor {
    async fn handle_event(&mut self, message: AggregatorMessage) {
        debug!("Got aggregator message");
        match message {
            AggregatorMessage::StartSearch(answer, mut position, mut position_history) => {
                let cancellation_rx = self.cancellation_rx.clone();
                let (result_tx, mut result_rx) = mpsc::unbounded_channel();
                // Should end by itself after cancellation or dropping of the move receiver
                let _search_task = std::thread::spawn(move || {
                    let mut searcher =
                        Searcher::new(&mut position_history, &mut position, cancellation_rx);
                    searcher.search(result_tx)
                });

                let mut result = None;

                while let Some(r) = result_rx.recv().await {
                    result = Some(r)
                }

                info!("Best move found: {:?}", result);

                let _ = answer.send(result);
            }
        }
    }

    fn new(
        receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
        cancellation_rx: watch::Receiver<()>,
    ) -> Self {
        Self {
            receiver,
            cancellation_rx,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }
}
