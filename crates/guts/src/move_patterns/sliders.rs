use crate::bitboard::Bitboard;
use crate::move_patterns::{generate, GenerateInput};
use crate::square::Square;
use std::collections::HashMap;

// TODO magic bitboards

pub struct BishopMovePatterns {
    map: [Bitboard; 64],
}

impl BishopMovePatterns {
    pub fn new() -> Self {
        let map = generate(|GenerateInput { dr, df, .. }| (dr.abs() == df.abs()));
        Self { map }
    }

    pub fn get_move(&self, s: &Square) -> Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map[s.bitboard_index()]
    }
}

pub struct RookMovePatterns {
    map: [Bitboard; 64],
}

impl RookMovePatterns {
    pub fn new() -> Self {
        let map =
            generate(|GenerateInput { dr, df, .. }| (dr == 0 && df != 0) || (dr != 0 && df == 0));
        Self { map }
    }

    pub fn get_move(&self, s: &Square) -> Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map[s.bitboard_index()]
    }
}

pub struct QueenMovePatterns {
    map: [Bitboard; 64],
}

impl QueenMovePatterns {
    pub fn new() -> Self {
        let map = generate(|GenerateInput { dr, df, .. }| {
            (dr.abs() == df.abs()) || (dr == 0 && df != 0) || (dr != 0 && df == 0)
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
    fn check_some_squares_rook() {
        let m = RookMovePatterns::new();

        let starting_square = Square::new(File::A, Rank::R1);
        let expected_squares = vec![
            Square::new(File::A, Rank::R2),
            Square::new(File::A, Rank::R3),
            Square::new(File::A, Rank::R4),
            Square::new(File::A, Rank::R5),
            Square::new(File::A, Rank::R6),
            Square::new(File::A, Rank::R7),
            Square::new(File::A, Rank::R8),
            Square::new(File::B, Rank::R1),
            Square::new(File::C, Rank::R1),
            Square::new(File::D, Rank::R1),
            Square::new(File::E, Rank::R1),
            Square::new(File::F, Rank::R1),
            Square::new(File::G, Rank::R1),
            Square::new(File::H, Rank::R1),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_move_count_rook() {
        let m = RookMovePatterns::new();
        for s in Square::ALL.iter() {
            let res = m.get_move(s); // should not panic
            assert_eq!(res.num_set(), 14)
        }
    }

    #[test]
    fn check_some_squares_bishop() {
        let m = BishopMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let expected_squares = vec![
            Square::new(File::A, Rank::R1),
            Square::new(File::C, Rank::R3),
            Square::new(File::D, Rank::R4),
            Square::new(File::E, Rank::R5),
            Square::new(File::F, Rank::R6),
            Square::new(File::G, Rank::R7),
            Square::new(File::H, Rank::R8),
            Square::new(File::C, Rank::R1),
            Square::new(File::A, Rank::R3),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics_bishop() {
        let m = BishopMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(s); // should not panic
        }
    }

    #[test]
    fn check_some_squares_queen() {
        let m = QueenMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let expected_squares = vec![
            Square::new(File::A, Rank::R1),
            Square::new(File::C, Rank::R3),
            Square::new(File::D, Rank::R4),
            Square::new(File::E, Rank::R5),
            Square::new(File::F, Rank::R6),
            Square::new(File::G, Rank::R7),
            Square::new(File::H, Rank::R8),
            Square::new(File::C, Rank::R1),
            Square::new(File::A, Rank::R3),
            Square::new(File::B, Rank::R1),
            Square::new(File::B, Rank::R3),
            Square::new(File::B, Rank::R4),
            Square::new(File::B, Rank::R5),
            Square::new(File::B, Rank::R6),
            Square::new(File::B, Rank::R7),
            Square::new(File::B, Rank::R8),
            Square::new(File::A, Rank::R2),
            Square::new(File::C, Rank::R2),
            Square::new(File::D, Rank::R2),
            Square::new(File::E, Rank::R2),
            Square::new(File::F, Rank::R2),
            Square::new(File::G, Rank::R2),
            Square::new(File::H, Rank::R2),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics_queen() {
        let m = QueenMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(s); // should not panic
        }
    }
}
