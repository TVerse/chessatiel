use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::move_patterns::{generate, GenerateInput};
use crate::rank::Rank;
use crate::square::Square;

pub struct PawnMovePatterns {
    moves: [Bitboard; 64],
    captures: [Bitboard; 64],
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

    pub fn get_move(&self, s: &Square) -> Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.moves[s.bitboard_index()]
    }

    pub fn get_capture(&self, s: &Square) -> Bitboard {
        // TODO unwrap should be safe, all starting boards must exist here
        self.captures[s.bitboard_index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;
    use crate::square::Square;

    #[test]
    fn check_some_squares_initial_move() {
        let m = PawnMovePatterns::new(Color::White);

        let starting_square = Square::new(File::A, Rank::R2);
        let expected_squares = vec![
            Square::new(File::A, Rank::R3),
            Square::new(File::A, Rank::R4),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn check_some_squares_after_second_move() {
        let m = PawnMovePatterns::new(Color::White);

        let starting_square = Square::new(File::A, Rank::R3);
        let expected_squares = vec![Square::new(File::A, Rank::R4)];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn check_some_squares_initial_move_black() {
        let m = PawnMovePatterns::new(Color::Black);

        let starting_square = Square::new(File::A, Rank::R7);
        let expected_squares = vec![
            Square::new(File::A, Rank::R6),
            Square::new(File::A, Rank::R5),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn check_some_squares_after_second_move_black() {
        let m = PawnMovePatterns::new(Color::Black);

        let starting_square = Square::new(File::A, Rank::R6);
        let expected_squares = vec![Square::new(File::A, Rank::R5)];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn check_some_captures_white() {
        let m = PawnMovePatterns::new(Color::White);
        let starting_square = Square::new(File::D, Rank::R4);
        let expected_squares = vec![
            Square::new(File::C, Rank::R5),
            Square::new(File::E, Rank::R5),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_capture(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn check_some_captures_black() {
        let m = PawnMovePatterns::new(Color::Black);
        let starting_square = Square::new(File::D, Rank::R4);
        let expected_squares = vec![
            Square::new(File::C, Rank::R3),
            Square::new(File::E, Rank::R3),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_capture(&starting_square);

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics() {
        let m = PawnMovePatterns::new(Color::White);
        for s in Square::ALL.iter() {
            let _res = m.get_move(s); // should not panic
            let _res = m.get_capture(s); // should not panic
        }

        let m = PawnMovePatterns::new(Color::Black);
        for s in Square::ALL.iter() {
            let _res = m.get_move(s); // should not panic
            let _res = m.get_capture(s); // should not panic
        }
    }
}
