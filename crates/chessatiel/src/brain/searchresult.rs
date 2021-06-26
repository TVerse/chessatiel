use crate::brain::ResultInfo;
use guts::Move;

#[derive(Debug, Clone)]
pub struct SearchResult {
    chess_move: Move,
    result_info: ResultInfo,
}

impl SearchResult {
    pub fn new(chess_move: Move, result_info: ResultInfo) -> Self {
        Self {
            chess_move,
            result_info,
        }
    }

    pub fn chess_move(&self) -> &Move {
        &self.chess_move
    }
}
