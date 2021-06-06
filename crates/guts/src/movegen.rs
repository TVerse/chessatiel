use crate::chess_move::ExtraMoveInfo;
use crate::{BaseMovePatterns, Gamestate, Move, Piece};

pub struct MoveGenerator {
    move_patterns: BaseMovePatterns,
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            move_patterns: BaseMovePatterns::new(),
        }
    }

    pub fn generate_legal_moves_for(
        &self,
        gamestate: &Gamestate,
    ) -> impl Iterator<Item = Move> + '_ {
        self.generate_pawn_moves(gamestate)
            .chain(self.generate_knight_moves(gamestate))
            .chain(self.generate_bishop_moves(gamestate))
            .chain(self.generate_rook_moves(gamestate))
            .chain(self.generate_queen_moves(gamestate))
            .chain(self.generate_king_moves(gamestate))
    }

    fn generate_pawn_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_pawns = gamestate.board()[gamestate.active_color()][Piece::Pawn];
        let pawn_moves = self.move_patterns.pawn(gamestate.active_color());
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();

        own_pawns.into_iter().flat_map(move |from| {
            let moves = pawn_moves.get_move(&from) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::Pawn))) //, MoveNote::StandardMove)
            })
        })
    }

    fn generate_knight_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_knights = gamestate.board()[gamestate.active_color()][Piece::Knight];
        let knight_moves = self.move_patterns.knight();
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();

        own_knights.into_iter().flat_map(move |from| {
            let moves = knight_moves.get_move(&from) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::Knight))) //, MoveNote::StandardMove)
            })
        })
    }

    fn generate_bishop_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_bishops = gamestate.board()[gamestate.active_color()][Piece::Bishop];
        let bishop_moves = self.move_patterns.bishop();
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();
        let all_pieces = gamestate.board()[!gamestate.active_color()].all_pieces() | own_pieces;

        own_bishops.into_iter().flat_map(move |from| {
            let moves = bishop_moves.get_move(&from, &all_pieces) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::Bishop))) //, MoveNote::StandardMove)
            })
        })
    }

    fn generate_rook_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_rooks = gamestate.board()[gamestate.active_color()][Piece::Rook];
        let rook_moves = self.move_patterns.rook();
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();
        let all_pieces = gamestate.board()[!gamestate.active_color()].all_pieces() | own_pieces;

        own_rooks.into_iter().flat_map(move |from| {
            let moves = rook_moves.get_move(&from, &all_pieces) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::Rook))) //, MoveNote::StandardMove)
            })
        })
    }

    fn generate_queen_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_queens = gamestate.board()[gamestate.active_color()][Piece::Queen];
        let queen_moves = self.move_patterns.queen();
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();
        let all_pieces = gamestate.board()[!gamestate.active_color()].all_pieces() | own_pieces;

        own_queens.into_iter().flat_map(move |from| {
            let moves = queen_moves.get_move(&from, &all_pieces) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::Queen))) //, MoveNote::StandardMove)
            })
        })
    }

    fn generate_king_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_kings = gamestate.board()[gamestate.active_color()][Piece::King];
        let king_moves = self.move_patterns.king();
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();

        own_kings.into_iter().flat_map(move |from| {
            let moves = king_moves.get_move(&from) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Some(ExtraMoveInfo::new(Piece::King))) //, MoveNote::StandardMove)
            })
        })
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

    fn compare_moves<F>(starting_state_fen: &'static str, filter: F, expected: &mut [Move])
    where
        F: FnMut(&Move) -> bool,
    {
        let generator = MoveGenerator::new();

        let starting_state = Gamestate::from_str(starting_state_fen).unwrap();

        let mut moves: Vec<_> = generator
            .generate_legal_moves_for(&starting_state)
            .filter(filter)
            .collect();

        moves.sort();

        expected.sort();

        let pretty_error = pretty_error(moves.as_slice(), expected);

        assert_eq!(moves, expected, "\n{}", pretty_error)
    }

    #[test]
    fn movegen_queen() {
        compare_moves(
            "rnbqkbnr/pppppppp/8/8/8/2PP4/PP2PPPP/RNBQKBNR w KQkq - 0 1",
            |m| m.extra.as_ref().unwrap().piece == Piece::Queen,
            vec![
                Move::from_str("d1d2").unwrap(),
                Move::from_str("d1c2").unwrap(),
                Move::from_str("d1b3").unwrap(),
                Move::from_str("d1a4").unwrap(),
            ]
            .as_mut_slice(),
        )
    }

    #[test]
    fn movegen_check() {
        compare_moves(
            "r7/8/8/8/8/8/7r/K7 w - - 0 1",
            |m| m.extra.as_ref().unwrap().piece == Piece::King,
            vec![Move::from_str("a1b1").unwrap()].as_mut_slice(),
        )
    }
}
