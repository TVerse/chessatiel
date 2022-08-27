pub mod pst;

use crate::evaluator::Evaluator;
use crate::{CentipawnScore, PieceSquareTable, SHARED_COMPONENTS};
use guts::{Piece, Position};
use log::debug;
use std::collections::HashMap;

pub struct PieceSquareTableEvaluator<'a> {
    base_values: HashMap<Piece, i32>,
    pst: &'a PieceSquareTable,
}

impl Default for PieceSquareTableEvaluator<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceSquareTableEvaluator<'static> {
    pub fn new() -> Self {
        Self::with_pst(&SHARED_COMPONENTS.pst)
    }
}

impl<'a> PieceSquareTableEvaluator<'a> {
    pub fn with_pst(pst: &'a PieceSquareTable) -> Self {
        Self {
            base_values: HashMap::from([
                (Piece::Pawn, 100),
                (Piece::Bishop, 300),
                (Piece::Knight, 300),
                (Piece::Rook, 500),
                (Piece::Queen, 900),
                (Piece::King, 0),
            ]),
            pst,
        }
    }
}

impl Evaluator for PieceSquareTableEvaluator<'_> {
    fn evaluate(&self, position: &Position) -> CentipawnScore {
        let mut my_score = 0;
        for p in Piece::ALL {
            my_score += (position.board()[position.active_color()][p].count_ones() as i32)
                * self.base_values.get(&p).unwrap();
        }
        let mut their_score = 0;
        for p in Piece::ALL {
            their_score += (position.board()[!position.active_color()][p].count_ones() as i32)
                * self.base_values.get(&p).unwrap();
        }

        let pst_score = self.pst.get(position);
        debug!("Got PST score: {pst_score}");
        let pst_score = (pst_score * 10000.0) as i32;

        CentipawnScore(my_score - their_score + pst_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn piece_value_evaluator() {
        let pst = PieceSquareTable::zeroes();
        let position = Position::default();
        assert_eq!(
            PieceSquareTableEvaluator::with_pst(&pst).evaluate(&position),
            CentipawnScore::ZERO
        )
    }

    #[test]
    fn piece_value_evaluator_2() {
        let pst = PieceSquareTable::zeroes();
        let evaluator = PieceSquareTableEvaluator::with_pst(&pst);

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(-500));

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(500));
    }
}
