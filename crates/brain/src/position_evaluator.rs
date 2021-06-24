use crate::Centipawn;
use guts::{Piece, PieceBoard, Position};

pub struct PositionEvaluator {}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, position: &Position) -> Centipawn {
        let score = self.evaluate_piece_score(&position.board()[position.active_color()])
            - self.evaluate_piece_score(&position.board()[!position.active_color()]);
        Self::clamp(Centipawn(score))
    }

    fn evaluate_piece_score(&self, piece_board: &PieceBoard) -> i64 {
        piece_board[Piece::Pawn].count_ones() as i64 * 100
            + piece_board[Piece::Knight].count_ones() as i64 * 300
            + piece_board[Piece::Bishop].count_ones() as i64 * 300
            + piece_board[Piece::Rook].count_ones() as i64 * 500
            + piece_board[Piece::Queen].count_ones() as i64 * 900
    }

    fn clamp(i: Centipawn) -> Centipawn {
        if i > Centipawn::WIN {
            Centipawn::WIN
        } else if i < Centipawn::LOSS {
            Centipawn::LOSS
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

        assert_eq!(evaluator.evaluate(&position), Centipawn(0))
    }

    #[test]
    fn more_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("k7/8/8/8/8/8/8/KQ6 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Centipawn(900))
    }

    #[test]
    fn less_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("kq6/8/8/8/8/8/8/K7 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Centipawn(-900))
    }
}
