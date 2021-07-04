use crate::brain::Score;
use guts::Move;

#[derive(Debug, Clone)]
pub struct SearchResult {
    chess_move: Move,
    score: Score,
}

impl SearchResult {
    pub fn new(chess_move: Move, score: Score) -> Self {
        Self { chess_move, score }
    }

    pub fn chess_move(&self) -> &Move {
        &self.chess_move
    }

    pub fn score(&self) -> &Score {
        &self.score
    }
}
