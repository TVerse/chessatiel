use crate::brain::position_history::PositionHistory;
use crate::brain::{MoveResult, SHARED_COMPONENTS};
use guts::MoveBuffer;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub struct Searcher<'a> {
    position_history: &'a mut PositionHistory,
}

impl<'a> Searcher<'a> {
    pub fn new(position_history: &'a mut PositionHistory) -> Self {
        Self { position_history }
    }

    pub fn search(
        &mut self,
        output: mpsc::UnboundedSender<MoveResult>,
        _cancel: broadcast::Receiver<()>,
    ) {
        let mut buf = MoveBuffer::new();
        let _in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(self.position_history.current_position(), &mut buf);

        let m = match buf.moves.into_iter().next() {
            Some(m) => m,
            None => {
                return;
            }
        };

        let move_result = MoveResult {
            chess_move: m.clone(),
            pv: vec![m],
        };

        let _ = output.send(move_result);
    }
}
