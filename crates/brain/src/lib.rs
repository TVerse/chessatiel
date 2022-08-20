mod aggregator;
pub mod evaluator;
pub mod position_hash_history;
pub mod searcher;
mod time_manager;

use crate::aggregator::AggregatorHandle;
use crate::evaluator::CentipawnScore;
use crate::position_hash_history::PositionHashHistory;
use guts::{Color, Move, MoveBuffer, MoveGenerator, Position};
use log::info;
use once_cell::sync::Lazy;
use std::time::Duration;
use thiserror::Error;
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

#[derive(Debug, Default, Clone)]
pub struct SearchConfiguration {
    pub depth: Option<usize>,
    pub remaining_time: Option<RemainingTime>,
}

#[derive(Debug, Copy, Clone)]
pub enum RemainingTime {
    ForGame {
        remaining: Duration,
        increment: Duration,
    },
    ForMove(Duration),
}

#[derive(Debug)]
enum EngineMessage {
    SetInitialValues(AckTx, Position, Vec<String>),
    SetMoves(AckTx, Vec<String>),
    CurrentColor(AnswerTx<Color>),
    Go(
        AnswerTx<Result<AnswerRx<Option<MoveResult>>, EngineError>>,
        SearchConfiguration,
    ),
    Stop(AnswerTx<bool>),
}

static SHARED_COMPONENTS: Lazy<EngineSharedComponents> = Lazy::new(|| EngineSharedComponents {
    move_generator: MoveGenerator::new(),
});

#[derive(Debug)]
struct EngineSharedComponents {
    move_generator: MoveGenerator,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("calculation already in progress")]
    CalculationAlreadyInProgress,
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

    pub async fn set_initial_values(&self, position: Position, moves: Vec<String>) {
        let (tx, rx) = ack();
        let msg = EngineMessage::SetInitialValues(tx, position, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn set_moves(&self, moves: Vec<String>) {
        let (tx, rx) = ack();
        let msg = EngineMessage::SetMoves(tx, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn current_color(&self) -> Color {
        let (tx, rx) = answer();
        let msg = EngineMessage::CurrentColor(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn go(
        &self,
        search_configuration: SearchConfiguration,
    ) -> Result<AnswerRx<Option<MoveResult>>, EngineError> {
        let (tx, rx) = answer();
        let msg = EngineMessage::Go(tx, search_configuration);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn stop(&self) -> bool {
        let (tx, rx) = answer();
        let msg = EngineMessage::Stop(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

struct CurrentCalculation {
    join_handle: tokio::task::JoinHandle<()>,
    stop: oneshot::Sender<()>,
}

struct EngineActor {
    initial_position: Position,
    hash_history: PositionHashHistory,
    current_position: Position,
    receiver: mpsc::UnboundedReceiver<EngineMessage>,
    cancellation_rx: watch::Receiver<()>,
    current_calculation: Option<CurrentCalculation>,
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
            initial_position: Position::default(),
            hash_history: PositionHashHistory::new(Position::default().hash()),
            current_position: Position::default(),
            receiver,
            cancellation_rx,
            current_calculation: None,
        }
    }

    async fn handle_event(&mut self, message: EngineMessage) {
        match message {
            EngineMessage::SetInitialValues(ack, position, move_strings) => {
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
            EngineMessage::CurrentColor(answer) => {
                let _ = answer.send(self.current_position.active_color());
            }
            EngineMessage::Go(ans, config) => {
                let result = if !self.check_calculation_running() {
                    let (stop_tx, stop_rx) = oneshot::channel();
                    let (answer_tx, answer_rx) = answer();
                    let cancellation_rx = self.cancellation_rx.clone();
                    let pos = self.current_position.clone();
                    let history = self.hash_history.clone();
                    let join_handle = tokio::spawn(async {
                        let aggregator = AggregatorHandle::new(cancellation_rx);
                        let res = aggregator.start_search(stop_rx, pos, history, config).await;
                        let _ = answer_tx.send(res);
                    });
                    self.current_calculation = Some(CurrentCalculation {
                        join_handle,
                        stop: stop_tx,
                    });
                    Ok(answer_rx)
                } else {
                    Err(EngineError::CalculationAlreadyInProgress)
                };
                let _ = ans.send(result);
            }
            EngineMessage::Stop(answer) => {
                let result = if let Some(current_calculation) = self.current_calculation.take() {
                    if !current_calculation.join_handle.is_finished() {
                        let _ = current_calculation.stop.send(());
                        let _ = current_calculation.join_handle.await;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                let _ = answer.send(result);
            }
        }
    }

    fn check_calculation_running(&mut self) -> bool {
        if let Some(ref current_calculation) = self.current_calculation {
            info!("Found a calculation");
            if current_calculation.join_handle.is_finished() {
                self.current_calculation = None;
                false
            } else {
                info!("Calculation not done yet");
                true
            }
        } else {
            false
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
