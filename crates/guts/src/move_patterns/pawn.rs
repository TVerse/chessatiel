use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::move_patterns::{generate, GenerateInput};
use crate::rank::Rank;
use crate::square::Square;
use std::collections::HashMap;

pub struct PawnMovePatterns {
    moves: HashMap<Bitboard, Bitboard>,
    captures: HashMap<Bitboard, Bitboard>,
}

// Pawns on closest rank can move 1 square, pawns on furthest can not move at all.
impl PawnMovePatterns {
    pub fn new(color: Color) -> Self {
        let moves = generate(|GenerateInput { dr, df, from, .. }| {
            let starting_rank = match color {
                Color::White => Rank::R2,
                Color::Black => Rank::R7,
            };

            let from_rank = from.rank();

            df == 0
                && match color {
                    Color::White => {
                        if from_rank == starting_rank {
                            dr == 1 || dr == 2
                        } else {
                            dr == 1
                        }
                    }
                    Color::Black => {
                        if from_rank == starting_rank {
                            dr == -1 || dr == -2
                        } else {
                            dr == -1
                        }
                    }
                }
        });
        let captures = generate(|GenerateInput { dr, df, .. }| {
            df.abs() == 1
                && match color {
                    Color::White => dr == 1,
                    Color::Black => dr == -1,
                }
        });
        Self { moves, captures }
    }

    pub fn get_move(&self, bb: &Bitboard) -> &Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.moves.get(bb).unwrap()
    }

    pub fn get_capture(&self, bb: &Bitboard) -> &Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.captures.get(bb).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn check_some_squares_initial_move() {
        let m = PawnMovePatterns::new(Color::White);

        let starting_square = Square::new(File::A, Rank::R2);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![
            Square::new(File::A, Rank::R3),
            Square::new(File::A, Rank::R4),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn check_some_squares_after_second_move() {
        let m = PawnMovePatterns::new(Color::White);

        let starting_square = Square::new(File::A, Rank::R3);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![Square::new(File::A, Rank::R4)];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn check_some_squares_initial_move_black() {
        let m = PawnMovePatterns::new(Color::Black);

        let starting_square = Square::new(File::A, Rank::R7);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![
            Square::new(File::A, Rank::R6),
            Square::new(File::A, Rank::R5),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn check_some_squares_after_second_move_black() {
        let m = PawnMovePatterns::new(Color::Black);

        let starting_square = Square::new(File::A, Rank::R6);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![Square::new(File::A, Rank::R5)];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn check_some_captures_white() {
        let m = PawnMovePatterns::new(Color::White);
        let starting_square = Square::new(File::D, Rank::R4);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![
            Square::new(File::C, Rank::R5),
            Square::new(File::E, Rank::R5),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_capture(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn check_some_captures_black() {
        let m = PawnMovePatterns::new(Color::Black);
        let starting_square = Square::new(File::D, Rank::R4);
        let starting_board = Bitboard::from_square(&starting_square);
        let expected_squares = vec![
            Square::new(File::C, Rank::R3),
            Square::new(File::E, Rank::R3),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_capture(&starting_board);

        assert_eq!(result, &expected_board)
    }

    #[test]
    fn get_never_panics() {
        let m = PawnMovePatterns::new(Color::White);
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(&bb); // should not panic
            let _res = m.get_capture(&bb); // should not panic
        }

        let m = PawnMovePatterns::new(Color::Black);
        for s in Square::ALL.iter() {
            let bb = Bitboard::from_square(s);
            let _res = m.get_move(&bb); // should not panic
            let _res = m.get_capture(&bb); // should not panic
        }
    }
}
