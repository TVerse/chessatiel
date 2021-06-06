use crate::bitboard::Bitboard;
use crate::move_patterns::{generate, GenerateInput};
use crate::square::Square;
use std::collections::HashMap;

pub struct KnightMovePatterns {
    map: [Bitboard; 64],
}

impl KnightMovePatterns {
    pub fn new() -> Self {
        let map = generate(|GenerateInput { dr, df, .. }| {
            (dr.abs() == 2 && df.abs() == 1) || (dr.abs() == 1 && df.abs() == 2)
        });
        Self { map }
    }

    pub fn get_move(&self, s: &Square) -> Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map[s.bitboard_index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;
    use crate::square::Square;

    #[test]
    fn check_some_squares() {
        let km = KnightMovePatterns::new();

        let starting_square = Square::new(File::A, Rank::R1);
        let expected_squares = vec![
            Square::new(File::B, Rank::R3),
            Square::new(File::C, Rank::R2),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = km.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics() {
        let km = KnightMovePatterns::new();
        for s in Square::ALL.iter() {
            let _res = km.get_move(s); // should not panic
        }
    }
}
