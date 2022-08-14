use crate::evaluator::Evaluator;
use crate::CentipawnScore;
use guts::{Piece, Position};
use std::collections::HashMap;

pub struct PieceValueEvaluator {
    values: HashMap<Piece, i32>,
}

impl Default for PieceValueEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceValueEvaluator {
    pub fn new() -> Self {
        Self {
            values: HashMap::from([
                (Piece::Pawn, 100),
                (Piece::Bishop, 300),
                (Piece::Knight, 300),
                (Piece::Rook, 500),
                (Piece::Queen, 900),
                (Piece::King, 0),
            ]),
        }
    }
}

impl Evaluator for PieceValueEvaluator {
    fn evaluate(&self, position: &Position) -> CentipawnScore {
        let mut my_score = 0;
        for p in Piece::ALL {
            my_score += (position.board()[position.active_color()][p].count_ones() as i32)
                * self.values.get(&p).unwrap();
        }
        let mut their_score = 0;
        for p in Piece::ALL {
            their_score += (position.board()[!position.active_color()][p].count_ones() as i32)
                * self.values.get(&p).unwrap();
        }

        CentipawnScore(my_score - their_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn piece_value_evaluator() {
        let position = Position::default();
        assert_eq!(
            PieceValueEvaluator::new().evaluate(&position),
            CentipawnScore::ZERO
        )
    }

    #[test]
    fn piece_value_evaluator_2() {
        let evaluator = PieceValueEvaluator::new();

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(-500));

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(500));
    }
}
