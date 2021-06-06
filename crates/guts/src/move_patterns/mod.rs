use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::move_patterns::king::KingMovePatterns;
use crate::move_patterns::knight::KnightMovePatterns;
use crate::move_patterns::pawn::PawnMovePatterns;
use crate::move_patterns::sliders::{BishopMovePatterns, QueenMovePatterns, RookMovePatterns};
use crate::square::Square;
use std::collections::HashMap;

mod king;
mod knight;
mod pawn;
mod sliders;

/*
Implementation comments:
* Abstract over map generation? Procedure is always the same
 */

pub struct BaseMovePatterns {
    pawn_white: PawnMovePatterns,
    pawn_black: PawnMovePatterns,
    knight: KnightMovePatterns,
    bishop: BishopMovePatterns,
    rook: RookMovePatterns,
    queen: QueenMovePatterns,
    king: KingMovePatterns,
}

impl BaseMovePatterns {
    pub fn new() -> Self {
        Self {
            pawn_white: PawnMovePatterns::new(Color::White),
            pawn_black: PawnMovePatterns::new(Color::Black),
            knight: KnightMovePatterns::new(),
            bishop: BishopMovePatterns::new(),
            rook: RookMovePatterns::new(),
            queen: QueenMovePatterns::new(),
            king: KingMovePatterns::new(),
        }
    }

    pub fn pawn(&self, c: Color) -> &PawnMovePatterns {
        match c {
            Color::White => &self.pawn_white,
            Color::Black => &self.pawn_black,
        }
    }

    pub fn knight(&self) -> &KnightMovePatterns {
        &self.knight
    }

    pub fn king(&self) -> &KingMovePatterns {
        &self.king
    }
}

struct GenerateInput<'a> {
    dr: i16,
    df: i16,
    from: &'a Square,
    to: &'a Square,
}

fn generate<'a, P: Fn(GenerateInput<'a>) -> bool>(p: P) -> [Bitboard; 64] {
    let mut map = [Bitboard(0); 64];
    for from in Square::ALL.iter() {
        let from_rank = from.rank().index() as i16;
        let from_file = from.file().index() as i16;
        let to = Square::ALL.iter().filter(|&to| {
            let to_rank = to.rank().index() as i16;
            let to_file = to.file().index() as i16;

            let dr = to_rank - from_rank;
            let df = to_file - from_file;

            let gi = GenerateInput { dr, df, from, to };

            (from != to) && p(gi)
        });
        let bb = Bitboard::from_squares_ref(to);
        map[from.bitboard_index()] = bb
    }
    map
}
