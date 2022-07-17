mod evaluator;
mod position_manager;
mod searcher;

use crate::brain::evaluator::CentipawnScore;
use crate::brain::position_manager::PositionHistory;
use crate::brain::searcher::Searcher;
use crate::{AckTx, AnswerTx, Shutdown};
use guts::{Color, Move, MoveGenerator, Position};
use log::debug;
use log::warn;
use once_cell::sync::Lazy;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum MoveResult {
    BestMove {
        chess_move: Move,
        score: CentipawnScore,
    },
    // ForcedMate{in_moves: i32, m: Move},
    GameAlreadyFinished,
}

#[derive(Debug)]
pub enum EngineCommand {
    SetInitialValues(AckTx, Color, Position, Vec<String>),
    SetMoves(AckTx, Vec<String>),
    IsMyMove(AnswerTx<bool>),
    Go(AnswerTx<MoveResult>, bool),
}

static SHARED_COMPONENTS: Lazy<EngineSharedComponents> = Lazy::new(|| EngineSharedComponents {
    move_generator: MoveGenerator::new(),
});

#[derive(Debug)]
struct EngineSharedComponents {
    move_generator: MoveGenerator,
}

#[derive(Debug)]
pub struct Engine {
    position_history: PositionHistory,
    my_color: Color,
    rx: mpsc::Receiver<EngineCommand>,
    searcher: Searcher,
}

impl Engine {
    fn new(rx: mpsc::Receiver<EngineCommand>) -> Self {
        Lazy::force(&SHARED_COMPONENTS);
        Self {
            position_history: PositionHistory::default(),
            my_color: Color::White,
            rx,
            searcher: Searcher::new(),
        }
    }

    pub(crate) fn start(shutdown: Shutdown, rx: mpsc::Receiver<EngineCommand>) -> JoinHandle<bool> {
        tokio::spawn(async move { Engine::new(rx).run(shutdown).await })
    }

    pub(crate) async fn run(self, mut shutdown: Shutdown) -> bool {
        select! {
            _ = self.handle_events() => {
                warn!("Engine event stream stopped without receiving kill signal!");
                false
            }
            _ = shutdown.recv() => {
                debug!("Engine received kill signal");
                true
            }
        }
    }

    async fn handle_events(mut self) {
        while let Some(event) = self.rx.recv().await {
            self.handle_engine_event(event).await
        }
    }

    async fn handle_engine_event(&mut self, event: EngineCommand) {
        match event {
            EngineCommand::SetInitialValues(ack, my_color, position, moves) => {
                self.position_history = PositionHistory::new(position);
                self.position_history
                    .set_moves_from_strings(&moves, &SHARED_COMPONENTS.move_generator);
                self.my_color = my_color;
                ack.send(()).unwrap()
            }
            EngineCommand::SetMoves(ack, moves) => {
                self.position_history
                    .set_moves_from_strings(&moves, &SHARED_COMPONENTS.move_generator);
                ack.send(()).unwrap()
            }
            EngineCommand::IsMyMove(answer) => answer
                .send(self.my_color == self.position_history.current_position().active_color())
                .unwrap(),
            EngineCommand::Go(answer, _is_first_move) => answer
                .send(self.searcher.search(&self.position_history))
                .unwrap(),
        }
    }
}
