mod aggregator;
pub mod evaluator;
pub mod position_hash_history;
pub mod priority_buffer;
pub mod searcher;
pub mod statistics;
mod time_manager;
pub mod transposition_table;

use crate::aggregator::AggregatorHandle;
use crate::evaluator::main_evaluator::pst::PieceSquareTable;
use crate::evaluator::CentipawnScore;
use crate::position_hash_history::PositionHashHistory;
use guts::{BasicMoveBuffer, Color, Move, MoveGenerator, Position};
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
    pub depth: Option<u16>,
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
        AnswerTx<Result<mpsc::UnboundedReceiver<EngineUpdate>, EngineError>>,
        SearchConfiguration,
    ),
    Stop(AnswerTx<bool>),
}

static PST_DATA: &[u8] = include_bytes!("../resources/pst.bincode");

static SHARED_COMPONENTS: Lazy<EngineSharedComponents> = Lazy::new(|| {
    let pst = PieceSquareTable::from_bincode(PST_DATA);
    EngineSharedComponents {
        move_generator: MoveGenerator::new(),
        pst,
    }
});

#[derive(Debug)]
struct EngineSharedComponents {
    move_generator: MoveGenerator,
    pst: PieceSquareTable,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("calculation already in progress")]
    CalculationAlreadyInProgress,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum EngineUpdate {
    BestMove(MoveResult),
    Info {
        nps: Option<u64>,
        depth: Option<u64>,
        nodes: Option<u64>,
        tt_hits: Option<u64>,
        score: Option<i32>,
    },
}

#[derive(Clone)]
pub struct EngineHandle {
    sender: mpsc::UnboundedSender<EngineMessage>,
}

impl EngineHandle {
    pub fn new(cancellation_rx: watch::Receiver<()>) -> Self {
        Lazy::force(&SHARED_COMPONENTS);
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
    ) -> Result<mpsc::UnboundedReceiver<EngineUpdate>, EngineError> {
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
    aggregator: AggregatorHandle,
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
        let agg_cancel_rx = cancellation_rx.clone();
        Self {
            initial_position: Position::default(),
            hash_history: PositionHashHistory::new(Position::default().hash()),
            current_position: Position::default(),
            receiver,
            cancellation_rx,
            current_calculation: None,
            aggregator: AggregatorHandle::new(agg_cancel_rx),
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
                    let (updates_tx, updates_rx) = mpsc::unbounded_channel();
                    let pos = self.current_position.clone();
                    let history = self.hash_history.clone();
                    let agg = self.aggregator.clone();
                    let join_handle = tokio::spawn(async move {
                        agg.start_search(stop_rx, pos, history, config, updates_tx)
                            .await;
                    });
                    self.current_calculation = Some(CurrentCalculation {
                        join_handle,
                        stop: stop_tx,
                    });
                    Ok(updates_rx)
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
        moves.iter().for_each(|m| {
            let mut buf = BasicMoveBuffer::new();
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
