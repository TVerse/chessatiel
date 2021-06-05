use crate::bitboard::Bitboard;
use crate::move_patterns::{generate, GenerateInput};
use crate::square::Square;
use std::collections::HashMap;

pub struct BishopMovePatterns {
    map: HashMap<Bitboard, Bitboard>,
}

impl BishopMovePatterns {
    pub fn new() -> Self {
        let map = generate(|GenerateInput { dr, df, .. }| (dr.abs() == df.abs()));
        Self { map }
    }

    pub fn get_move(&self, bb: &Bitboard) -> &Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map.get(bb).unwrap()
    }
}

pub struct RookMovePatterns {
    map: HashMap<Bitboard, Bitboard>,
}

impl RookMovePatterns {
    pub fn new() -> Self {
        let map =
            generate(|GenerateInput { dr, df, .. }| (dr == 0 && df != 0) || (dr != 0 && df == 0));
        Self { map }
    }

    pub fn get_move(&self, bb: &Bitboard) -> &Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.map.get(bb).unwrap()
    }
}

pub struct QueenMovePatterns {
    map: HashMap<Bitboard, Bitboard>,
}

impl QueenMovePatterns {
    pub fn new() -> Self {
        let map = generate(|GenerateInput { dr, df, .. }| {
            (dr.abs() == df.abs()) || (dr == 0 && df != 0) || (dr != 0 && df == 0)
        });
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
    use crate::piece::Piece::Bishop;
    use crate::rank::Rank;

    #[test]
    fn check_some_squares_rook() {
        let m = RookMovePatterns::new();

        let starting_square = Square::new(File::A, Rank::R1);
        let starting_board = Bitboard::from_square(&starting_square);
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

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn get_move_count_rook() {
        let m = RookMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let res = m.get_move(&bb); // should not panic
            assert_eq!(res.num_set(), 14)
        }
    }

    #[test]
    fn check_some_squares_bishop() {
        let m = BishopMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let starting_board = Bitboard::from_square(&starting_square);
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

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn get_never_panics_bishop() {
        let m = BishopMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(&bb); // should not panic
        }
    }

    #[test]
    fn check_some_squares_queen() {
        let m = QueenMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let starting_board = Bitboard::from_square(&starting_square);
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

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn get_never_panics_queen() {
        let m = QueenMovePatterns::new();
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(&bb); // should not panic
        }
    }
}
