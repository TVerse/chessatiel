mod aggregator;
pub mod evaluator;
pub mod position_history;
pub mod searcher;

use crate::aggregator::AggregatorHandle;
use crate::evaluator::CentipawnScore;
use crate::position_history::PositionHistory;
use guts::{Color, Move, MoveGenerator, Position};
use once_cell::sync::Lazy;
use tokio::sync::{mpsc, oneshot};

type AnswerRx<T> = oneshot::Receiver<T>;
type AnswerTx<T> = oneshot::Sender<T>;
type AckRx = AnswerRx<()>;
type AckTx = AnswerTx<()>;

fn ack() -> (AckTx, AckRx) {
    oneshot::channel()
}

fn answer<T>() -> (AnswerTx<T>, AnswerRx<T>) {
    oneshot::channel()
}

#[derive(Debug, Clone)]
pub struct MoveResult {
    score: CentipawnScore,
    pv: Vec<Move>,
}

impl MoveResult {
    pub fn new(score: CentipawnScore, m: Move) -> Self {
        Self { score, pv: vec![m] }
    }

    pub fn score(&self) -> CentipawnScore {
        self.score
    }

    pub fn first_move(&self) -> &Move {
        self.pv.last().expect("Got empty MoveResult?")
    }

    pub fn _pv(&self) -> &[Move] {
        &self.pv
    }

    pub fn push(&mut self, m: Move) {
        self.pv.push(m)
    }

    pub fn invert_score(&mut self) {
        self.score = -self.score;
    }
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

impl Default for EngineHandle {
    fn default() -> Self {
        Self::new()
    }
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
    aggregator: AggregatorHandle,
    position_history: PositionHistory,
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
            aggregator: AggregatorHandle::new(),
            position_history: PositionHistory::new(Position::default()),
            my_color: Color::White,
            receiver,
        }
    }

    async fn handle_event(&mut self, message: EngineMessage) {
        match message {
            EngineMessage::SetInitialValues(ack, my_color, position, move_strings) => {
                self.my_color = my_color;
                self.position_history.reset_with(position);
                self.position_history
                    .set_moves_from_strings(&move_strings, &SHARED_COMPONENTS.move_generator);
                let _ = ack.send(());
            }
            EngineMessage::SetMoves(ack, move_strings) => {
                self.position_history
                    .set_moves_from_strings(&move_strings, &SHARED_COMPONENTS.move_generator);
                let _ = ack.send(());
            }
            EngineMessage::IsMyMove(answer) => {
                let _ = answer
                    .send(self.position_history.current_position().active_color() == self.my_color);
            }
            EngineMessage::Go(answer, _is_first) => {
                let res = self
                    .aggregator
                    .start_search(self.position_history.clone())
                    .await;
                let _ = answer.send(res);
            }
        }
    }
}
