mod aggregator;
pub mod evaluator;
pub mod position_hash_history;
pub mod searcher;

use crate::aggregator::AggregatorHandle;
use crate::evaluator::CentipawnScore;
use crate::position_hash_history::PositionHashHistory;
use guts::{Color, Move, MoveBuffer, MoveGenerator, Position};
use once_cell::sync::Lazy;
use tokio::select;
use tokio::sync::{mpsc, oneshot, watch};

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
    pub fn new(score: CentipawnScore) -> Self {
        Self {
            score,
            pv: Vec::new(),
        }
    }

    pub fn score(&self) -> CentipawnScore {
        self.score
    }

    pub fn first_move(&self) -> Option<&Move> {
        self.pv.last()
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

impl EngineHandle {
    pub fn new(cancellation_rx: watch::Receiver<()>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = EngineActor::new(receiver, cancellation_rx);
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
    initial_position: Position,
    hash_history: PositionHashHistory,
    current_position: Position,
    my_color: Color,
    receiver: mpsc::UnboundedReceiver<EngineMessage>,
    cancellation_rx: watch::Receiver<()>,
}

impl EngineActor {
    async fn run(&mut self) {
        while let Some(msg) = select! {
            Some(r) = self.receiver.recv() => Some(r),
            _ = self.cancellation_rx.changed() => None,
        } {
            self.handle_event(msg).await;
        }
    }

    fn new(
        receiver: mpsc::UnboundedReceiver<EngineMessage>,
        cancellation_rx: watch::Receiver<()>,
    ) -> Self {
        Lazy::force(&SHARED_COMPONENTS);
        Self {
            aggregator: AggregatorHandle::new(cancellation_rx.clone()),
            initial_position: Position::default(),
            hash_history: PositionHashHistory::new(Position::default().hash()),
            current_position: Position::default(),
            my_color: Color::White,
            receiver,
            cancellation_rx,
        }
    }

    async fn handle_event(&mut self, message: EngineMessage) {
        match message {
            EngineMessage::SetInitialValues(ack, my_color, position, move_strings) => {
                self.my_color = my_color;
                self.initial_position = position.clone();
                self.hash_history = PositionHashHistory::new(position.hash());
                self.current_position = position;
                self.set_from_strings(&move_strings);
                let _ = ack.send(());
            }
            EngineMessage::SetMoves(ack, move_strings) => {
                self.current_position = self.initial_position.clone();
                self.set_from_strings(&move_strings);
                let _ = ack.send(());
            }
            EngineMessage::IsMyMove(answer) => {
                let _ = answer.send(self.current_position.active_color() == self.my_color);
            }
            EngineMessage::Go(answer, _is_first) => {
                let res = self
                    .aggregator
                    .start_search(self.current_position.clone(), self.hash_history.clone())
                    .await;
                let _ = answer.send(res);
            }
        }
    }

    fn set_from_strings(&mut self, moves: &[String]) {
        let mut buf = MoveBuffer::new();
        moves.iter().for_each(|m| {
            let _ = SHARED_COMPONENTS
                .move_generator
                .generate_legal_moves_for(&self.current_position, &mut buf);

            let found_move = buf
                .iter()
                .find(|fm| &fm.as_uci() == m)
                .unwrap_or_else(|| panic!("Got invalid move {m}"));

            self.current_position.make_move(found_move);
            self.hash_history.push(self.current_position.hash());
        });
    }
}
