mod position_history;
mod searcher;

use crate::brain::position_history::PositionHistoryHandle;
use crate::brain::searcher::SearcherHandle;
use crate::{ack, answer, AckTx, AnswerTx};
use guts::{Color, Move, MoveGenerator, Position};
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct MoveResult {
    pub chess_move: Move,
    // pub score: CentipawnScore,
    pub pv: Vec<Move>,
}

#[derive(Debug)]
enum EngineMessage {
    SetInitialValues(AckTx, Color, Position, Vec<String>),
    SetMoves(AckTx, Vec<String>),
    IsMyMove(AnswerTx<bool>),
    Go(AnswerTx<Option<MoveResult>>, bool),
}

static SHARED_COMPONENTS: Lazy<EngineSharedComponents> = Lazy::new(|| EngineSharedComponents {
    move_generator: MoveGenerator::new(),
});

#[derive(Debug)]
struct EngineSharedComponents {
    move_generator: MoveGenerator,
}

#[derive(Clone)]
pub struct EngineHandle {
    sender: mpsc::UnboundedSender<EngineMessage>,
}

impl EngineHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = EngineActor::new(receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn set_initial_values(&self, color: Color, position: Position, moves: Vec<String>) {
        let (tx, rx) = ack();
        let msg = EngineMessage::SetInitialValues(tx, color, position, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn set_moves(&self, moves: Vec<String>) {
        let (tx, rx) = ack();
        let msg = EngineMessage::SetMoves(tx, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn is_my_move(&self) -> bool {
        let (tx, rx) = answer();
        let msg = EngineMessage::IsMyMove(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn go(&self, is_first_move: bool) -> Option<MoveResult> {
        let (tx, rx) = answer();
        let msg = EngineMessage::Go(tx, is_first_move);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct EngineActor {
    position_history: PositionHistoryHandle,
    searcher: SearcherHandle,
    my_color: Color,
    receiver: mpsc::UnboundedReceiver<EngineMessage>,
}

impl EngineActor {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }

    fn new(receiver: mpsc::UnboundedReceiver<EngineMessage>) -> Self {
        Lazy::force(&SHARED_COMPONENTS);
        Self {
            position_history: PositionHistoryHandle::new(),
            searcher: SearcherHandle::new(),
            my_color: Color::White,
            receiver,
        }
    }

    async fn handle_event(&mut self, message: EngineMessage) {
        match message {
            EngineMessage::SetInitialValues(ack, my_color, position, move_strings) => {
                self.my_color = my_color;
                self.position_history.reset_position(position).await;
                self.position_history.set_move_strings(move_strings).await;
                let _ = ack.send(());
            }
            EngineMessage::SetMoves(ack, move_strings) => {
                self.position_history.set_move_strings(move_strings).await;
                let _ = ack.send(());
            }
            EngineMessage::IsMyMove(answer) => {
                let _ = answer.send(
                    self.position_history
                        .get_current_position()
                        .await
                        .active_color()
                        == self.my_color,
                );
            }
            EngineMessage::Go(answer, _is_first) => {
                let res = self.searcher.search(self.position_history.clone()).await;
                let _ = answer.send(res);
            }
        }
    }
}
