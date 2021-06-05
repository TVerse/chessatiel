use crate::bitboard::Bitboard;
use crate::move_patterns::{generate, GenerateInput};
use std::collections::HashMap;

pub struct KnightMovePatterns {
    map: HashMap<Bitboard, Bitboard>,
}

impl KnightMovePatterns {
    pub fn new() -> Self {
        let map =
            generate(|GenerateInput { dr, df, .. }| (dr == 2 && df == 1) || (dr == 1 && df == 2));
        Self { map }
    }

    pub fn get_move(&self, bb: &Bitboard) -> &Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map.get(bb).unwrap()
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
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![
            Square::new(File::B, Rank::R3),
            Square::new(File::C, Rank::R2),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = km.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn get_never_panics() {
        let km = KnightMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = km.get_move(&bb); // should not panic
        }
    }
}
