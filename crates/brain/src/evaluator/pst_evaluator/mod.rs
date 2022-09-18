pub mod pst;

use crate::evaluator::Evaluator;
use crate::{CentipawnScore, PieceSquareTable, SHARED_COMPONENTS};
use guts::{Bitboard, Color, Piece, Position};
use log::debug;
use std::collections::HashMap;

pub struct PstEvaluator<'a> {
    base_values: HashMap<Piece, i32>,
    pst: &'a PieceSquareTable,
}

impl Default for PstEvaluator<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl PstEvaluator<'static> {
    pub fn new() -> Self {
        Self::with_pst(&SHARED_COMPONENTS.pst)
    }
}

impl<'a> PstEvaluator<'a> {
    const DOUBLED_PAWNS_WEIGHT: i32 = 50;

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

impl Evaluator for PstEvaluator<'_> {
    fn evaluate(&self, position: &Position) -> CentipawnScore {
        let mut score = 0;
        let my_color = position.active_color();
        for p in Piece::ALL {
            score += (position.board()[my_color][p].count_ones() as i32)
                * self.base_values.get(&p).unwrap();
        }
        for p in Piece::ALL {
            score -= (position.board()[!my_color][p].count_ones() as i32)
                * self.base_values.get(&p).unwrap();
        }

        score -= doubled_tripled_pawns(position.board()[my_color][Piece::Pawn], my_color)
            * Self::DOUBLED_PAWNS_WEIGHT;
        score += doubled_tripled_pawns(position.board()[!my_color][Piece::Pawn], !my_color)
            * Self::DOUBLED_PAWNS_WEIGHT;

        let pst_score = self.pst.get(position) * 1000.0;
        debug!("Got corrected PST score: {pst_score}");
        let pst_score = pst_score as i32;

        CentipawnScore(score + pst_score)
    }
}

fn doubled_tripled_pawns(pawns: Bitboard, color: Color) -> i32 {
    let pawns_in_front = pawns & pawns.front_span(color);
    let pawns_behind = pawns & pawns.rear_span(color);
    let front_and_behind = pawns_in_front & pawns_behind;
    let filled = front_and_behind.file_fill();
    let doubled = pawns_in_front & !filled;
    let at_least_triple = filled & Bitboard::new(0xFF);
    doubled.count_ones() as i32 + 2 * (at_least_triple.count_ones() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use guts::{File, Rank, Square};
    use std::str::FromStr;

    #[test]
    fn piece_value_evaluator() {
        let pst = PieceSquareTable::zeroes();
        let position = Position::default();
        assert_eq!(
            PstEvaluator::with_pst(&pst).evaluate(&position),
            CentipawnScore::ZERO
        )
    }

    #[test]
    fn piece_value_evaluator_2() {
        let pst = PieceSquareTable::zeroes();
        let evaluator = PstEvaluator::with_pst(&pst);

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(-500));

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(500));
    }

    #[test]
    fn test_doubled_pawns() {
        let bb = Bitboard::from_iter(
            [
                Square::new(File::A, Rank::R2),
                Square::new(File::A, Rank::R5),
                Square::new(File::B, Rank::R3),
            ]
            .into_iter(),
        );
        assert_eq!(doubled_tripled_pawns(bb, Color::White), 1);
        let bb = Bitboard::from_iter(
            [
                Square::new(File::A, Rank::R2),
                Square::new(File::A, Rank::R5),
                Square::new(File::B, Rank::R3),
                Square::new(File::B, Rank::R7),
            ]
            .into_iter(),
        );
        assert_eq!(doubled_tripled_pawns(bb, Color::White), 2);
        let bb = Bitboard::from_iter(
            [
                Square::new(File::A, Rank::R2),
                Square::new(File::A, Rank::R5),
                Square::new(File::A, Rank::R3),
            ]
            .into_iter(),
        );
        assert_eq!(doubled_tripled_pawns(bb, Color::White), 2);
        let bb = Bitboard::from_iter(
            [
                Square::new(File::A, Rank::R2),
                Square::new(File::A, Rank::R5),
                Square::new(File::A, Rank::R3),
                Square::new(File::B, Rank::R2),
                Square::new(File::B, Rank::R3),
            ]
            .into_iter(),
        );
        assert_eq!(doubled_tripled_pawns(bb, Color::White), 3);
    }
}
