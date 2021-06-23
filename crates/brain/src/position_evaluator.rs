use crate::Millipawn;
use guts::{Piece, PieceBoard, Position};

pub struct PositionEvaluator {}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, position: &Position) -> Millipawn {
        let score = self.evaluate_piece_score(&position.board()[position.active_color()])
            - self.evaluate_piece_score(&position.board()[!position.active_color()]);
        Self::clamp(Millipawn(score))
    }

    fn evaluate_piece_score(&self, piece_board: &PieceBoard) -> i64 {
        piece_board[Piece::Pawn].count_ones() as i64 * 1000
            + piece_board[Piece::Knight].count_ones() as i64 * 3000
            + piece_board[Piece::Bishop].count_ones() as i64 * 3000
            + piece_board[Piece::Rook].count_ones() as i64 * 5000
            + piece_board[Piece::Queen].count_ones() as i64 * 9000
    }

    fn clamp(i: Millipawn) -> Millipawn {
        if i > Millipawn::WIN {
            Millipawn::WIN
        } else if i < Millipawn::LOSS {
            Millipawn::LOSS
        } else {
            i
        }
    }
}

impl Default for PositionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn starting_position() {
        let evaluator = PositionEvaluator::new();
        let position = Position::default();

        assert_eq!(evaluator.evaluate(&position), Millipawn(0))
    }

    #[test]
    fn more_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("k7/8/8/8/8/8/8/KQ6 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Millipawn(9000))
    }

    #[test]
    fn less_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("kq6/8/8/8/8/8/8/K7 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Millipawn(-9000))
    }
}
