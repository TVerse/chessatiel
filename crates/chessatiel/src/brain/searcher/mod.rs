use crate::brain::position_history::PositionHistoryHandle;
use crate::brain::{MoveResult, SHARED_COMPONENTS};
use crate::{answer, AnswerTx};
use guts::MoveBuffer;
use tokio::sync::mpsc;

enum SearcherMessage {
    Search(AnswerTx<Option<MoveResult>>, PositionHistoryHandle),
}

#[derive(Clone)]
pub struct SearcherHandle {
    sender: mpsc::UnboundedSender<SearcherMessage>,
}

impl SearcherHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = SearcherActor::new(receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn search(&self, positions: PositionHistoryHandle) -> Option<MoveResult> {
        let (tx, rx) = answer();
        let msg = SearcherMessage::Search(tx, positions);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct SearcherActor {
    receiver: mpsc::UnboundedReceiver<SearcherMessage>,
}

impl SearcherActor {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await
        }
    }

    async fn handle_event(&mut self, message: SearcherMessage) {
        match message {
            SearcherMessage::Search(answer, position) => {
                let mut buf = MoveBuffer::new();
                let _in_check = SHARED_COMPONENTS
                    .move_generator
                    .generate_legal_moves_for(&position.get_current_position().await, &mut buf);

                let m = match buf.moves.into_iter().next() {
                    Some(m) => m,
                    None => {
                        let _ = answer.send(None);
                        return;
                    }
                };

                let _ = answer.send(Some(MoveResult {
                    chess_move: m.clone(),
                    pv: vec![m],
                }));
            }
        }
    }

    fn new(receiver: mpsc::UnboundedReceiver<SearcherMessage>) -> Self {
        Self { receiver }
    }
}
