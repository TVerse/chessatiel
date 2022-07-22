use crate::brain::position_history::PositionHistory;
use crate::brain::searcher::Searcher;
use crate::brain::MoveResult;
use crate::{answer, AnswerTx};
use log::{debug, info};
use tokio::sync::mpsc;
use tokio::sync::watch;

#[derive(Debug)]
enum AggregatorMessage {
    StartSearch(AnswerTx<Option<MoveResult>>, PositionHistory),
}

pub struct AggregatorHandle {
    sender: mpsc::UnboundedSender<AggregatorMessage>,
}

impl AggregatorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = AggregatorActor::new(receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start_search(&self, position_history: PositionHistory) -> Option<MoveResult> {
        let (tx, rx) = answer();
        let msg = AggregatorMessage::StartSearch(tx, position_history);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct AggregatorActor {
    receiver: mpsc::UnboundedReceiver<AggregatorMessage>,
    cancellation_tx: watch::Sender<()>,
}

impl AggregatorActor {
    async fn handle_event(&mut self, message: AggregatorMessage) {
        debug!("Got aggregator message");
        match message {
            AggregatorMessage::StartSearch(answer, mut position_history) => {
                let cancellation_rx = self.cancellation_tx.subscribe();
                let (result_tx, mut result_rx) = mpsc::unbounded_channel();
                // Should end by itself after cancellation or dropping of the move receiver
                let _search_task = std::thread::spawn(move || {
                    let mut searcher = Searcher::new(&mut position_history, cancellation_rx);
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

    fn new(receiver: mpsc::UnboundedReceiver<AggregatorMessage>) -> Self {
        let (cancellation_tx, _cancellation_rx) = watch::channel(());
        Self {
            receiver,
            cancellation_tx,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }
}

impl Drop for AggregatorActor {
    fn drop(&mut self) {
        let _ = self.cancellation_tx.send(());
    }
}
