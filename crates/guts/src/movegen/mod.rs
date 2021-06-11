mod king;
mod tables;

use crate::bitboard::Bitboard;
use crate::board::Sliders;
use crate::movegen::king::move_for_king;
use crate::movegen::tables::{KnightMovePatterns, SquaresBetween};
use crate::square::Square;
use crate::{Move, Piece, Position};

#[derive(Debug, Eq, PartialEq)]
struct Pin {
    pinner: Square,
    pinned: Square,
}

#[derive(Debug, Eq, PartialEq)]
struct KingSurroundings {
    checkers: Bitboard,
    pins: Vec<Pin>,
    king_danger: Bitboard,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Masks {
    king_danger: Bitboard,
    capture: Bitboard,
    push: Bitboard,
}

impl Masks {
    #[cfg(test)]
    pub const EMPTY: Masks = Masks::new(Bitboard::EMPTY, Bitboard::EMPTY, Bitboard::EMPTY);

    pub const fn new(king_danger: Bitboard, capture: Bitboard, push: Bitboard) -> Self {
        Self {
            king_danger,
            capture,
            push,
        }
    }
}

// TODO pull out commonly-used bitboards.
// TODO a lot of these methods don't involve knights and don't need self.
// TODO if the knight table can be const, this struct is obsolete.
pub struct MoveGenerator {
    knight_patterns: KnightMovePatterns,
    squares_between: SquaresBetween,
}

/*
Procedure: (https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/)

1. Find checking pieces.
  * More than one? Only legal moves are king moves.
2. Find king danger squares.
3. Generate capture and push masks. (If not in check, they are the full board). Difference cause en-passant.
4. Find pinned pieces and their legal moves, using masks.
5. Find other pieces' legal moves, using masks.
*/
impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            knight_patterns: KnightMovePatterns::new(),
            squares_between: SquaresBetween::new(),
        }
    }

    pub fn generate_legal_moves_for(&self, position: &Position) -> Vec<Move> {
        let KingSurroundings {
            checkers,
            pins,
            king_danger,
        } = self.king_surroundings(position);

        let num_checkers = checkers.count_ones();
        let own_pieces = &position.board()[position.active_color()];
        let own_king_sq = own_pieces[Piece::King]
            .first_set_square()
            .expect("No king?");

        let masks = if num_checkers == 1 {
            let checker_square = checkers.first_set_square().unwrap(); // Also only set square
            let piece = position.board().piece_at(checker_square).unwrap();
            if piece.is_slider() {
                Masks::new(
                    checkers,
                    own_king_sq.ray_between(checker_square),
                    king_danger,
                )
            } else {
                Masks::new(checkers, Bitboard::EMPTY, king_danger)
            }
        } else {
            Masks::new(Bitboard::EMPTY, Bitboard::EMPTY, king_danger)
        };

        let mut result = Vec::with_capacity(100); // TODO

        let king_moves = move_for_king(position, &masks);

        result.extend(king_moves);

        // Double check (or more), only king moves are possible.
        if num_checkers >= 2 {
            return result;
        }

        // result.extend(self.pinned_piece_moves(own_pieceboard, own_king_sq, pinners, pinned));

        result
    }

    fn king_surroundings(&self, position: &Position) -> KingSurroundings {
        let own_pieceboard = &position.board()[position.active_color()];
        let own_pieces = own_pieceboard.all_pieces();
        let own_king = own_pieceboard[Piece::King];
        let own_king_sq = own_king.first_set_square().unwrap();
        let opponent = !position.active_color();
        let enemy_pieceboard = &position.board()[opponent];
        let occupied = position.board().all_pieces();

        // Pawns and knights cannot pin
        let knight_check =
            self.knight_patterns.get_moves(own_king) & enemy_pieceboard[Piece::Knight];
        let enemy_pawns = enemy_pieceboard[Piece::Pawn];
        let pawn_check = (enemy_pawns.forward_left_one(opponent)
            | enemy_pawns.forward_right_one(opponent))
            & own_king;

        let enemy_cardinal = enemy_pieceboard[Piece::Rook] | enemy_pieceboard[Piece::Queen];
        let enemy_diagonal = enemy_pieceboard[Piece::Bishop] | enemy_pieceboard[Piece::Queen];

        // These two can probably be lookup tables?
        let cardinal_attackers = own_king.cardinal_attackers(Bitboard::FULL) & enemy_cardinal;
        let diagonal_attackers = own_king.diagonal_attackers(Bitboard::FULL) & enemy_diagonal;

        let attackers = cardinal_attackers | diagonal_attackers;

        // TODO vec size
        // TODO Use array instead to save the heap allocation, max amount of pinners is 8.
        // TODO I think these need to be paired so bitboards are not enough?
        let mut pins = Vec::with_capacity(5);
        let mut checkers = Bitboard::EMPTY;
        attackers.into_iter().for_each(|s| {
            let attacker = Bitboard::from_square(s);
            let pieces_between = self.squares_between.between(s, own_king_sq) & occupied;

            if pieces_between == Bitboard::EMPTY {
                // Nothing between? Check.
                checkers |= attacker
            } else if pieces_between.count_ones() == 1
                && (pieces_between & own_pieces).count_ones() == 1
            {
                // One piece between and it's ours? Pinned.
                let pinner = s;
                let pinned = pieces_between.first_set_square().unwrap();
                pins.push(Pin { pinner, pinned });
            } else {
                // Nothing
            }
        });

        let checkers = checkers | pawn_check | knight_check;

        KingSurroundings {
            checkers,
            pins,
            king_danger: self.king_danger(position),
        }
    }

    fn king_danger(&self, position: &Position) -> Bitboard {
        let opponent = !position.active_color();

        let Sliders { cardinal, diagonal } = position.board().sliders(opponent);
        let all_except_king =
            position.board().all_pieces() & !position.board()[position.active_color()][Piece::King];
        let empty = !all_except_king;

        let cardinal = cardinal.cardinal_attackers(empty);
        let diagonal = diagonal.diagonal_attackers(empty);

        let sliders = cardinal | diagonal;

        let opponent_pieces = &position.board()[opponent];
        let knights = opponent_pieces[Piece::Knight];
        let knights = self.knight_patterns.get_moves(knights);

        let pawns = opponent_pieces[Piece::Pawn];
        let pawns = pawns.forward_left_one(opponent) | pawns.forward_right_one(opponent);

        let kings = opponent_pieces[Piece::King];
        let kings = kings.surrounding();

        sliders | knights | pawns | kings
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;
    use crate::square::Square;
    use itertools::{EitherOrBoth, Itertools};
    use std::str::FromStr;

    fn pretty_error(moves: &[Move], expected: &[Move]) -> String {
        moves
            .iter()
            .zip_longest(expected.iter())
            .fold(String::new(), |acc, eob| {
                format!(
                    "{}{}\n",
                    acc,
                    match eob {
                        EitherOrBoth::Both(m, e) => format!("{} {}", m, e),
                        EitherOrBoth::Left(m) => format!("{} MISS", m),
                        EitherOrBoth::Right(e) => format!("MISS {}", e),
                    }
                )
            })
    }

    fn compare_moves<F>(starting_position_fen: &'static str, filter: F, expected: &mut [Move])
    where
        F: FnMut(&Move) -> bool,
    {
        let generator = MoveGenerator::new();

        let starting_position = Position::from_str(starting_position_fen).unwrap();

        let mut moves: Vec<_> = generator
            .generate_legal_moves_for(&starting_position)
            .into_iter()
            .filter(filter)
            .collect();

        moves.sort();

        expected.sort();

        let pretty_error = pretty_error(moves.as_slice(), expected);

        assert_eq!(moves, expected, "\n{}", pretty_error)
    }

    #[test]
    fn test_king_danger() {
        let generator = MoveGenerator::new();

        let position = Position::from_str("r7/8/8/8/8/8/1K6/8 w - - 0 1").unwrap();

        let expected =
            (Bitboard::A_FILE | Bitboard::RANK_8) & !(Bitboard::A_FILE & Bitboard::RANK_8);

        assert_eq!(generator.king_danger(&position), expected)
    }

    #[test]
    fn test_king_surroundings() {
        let generator = MoveGenerator::new();

        let position = Position::from_str("1r6/8/5q2/4P3/8/8/1K1B1r2/b4r2 w - - 0 1").unwrap();
        let surroundings = generator.king_surroundings(&position);

        let expected_checkers = Bitboard::from_squares(
            vec![
                Square::new(File::B, Rank::R8),
                Square::new(File::A, Rank::R1),
            ]
            .into_iter(),
        );

        let expected_pins = vec![
            Pin {
                pinner: Square::new(File::F, Rank::R2),
                pinned: Square::new(File::D, Rank::R2),
            },
            Pin {
                pinner: Square::new(File::F, Rank::R6),
                pinned: Square::new(File::E, Rank::R5),
            },
        ];

        assert_eq!(surroundings.checkers, expected_checkers);
        assert_eq!(surroundings.pins, expected_pins);
    }
}
