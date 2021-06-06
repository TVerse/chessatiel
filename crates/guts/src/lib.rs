pub use chess_move::Move;
pub use gamestate::Gamestate;
pub use move_patterns::BaseMovePatterns;
pub use parse_error::ParseError;

use crate::bitboard::Bitboard;
use crate::piece::Piece;
// use crate::chess_move::MoveNote;

mod bitboard;
mod board;
mod castling_rights;
mod chess_move;
mod color;
pub mod fen;
mod file;
mod gamestate;
mod move_patterns;
mod parse_error;
mod piece;
mod rank;
mod square;

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
    }

    fn generate_pawn_moves(&self, gamestate: &Gamestate) -> impl Iterator<Item = Move> + '_ {
        let own_pawns = gamestate.board()[gamestate.active_color()][Piece::Pawn];
        let pawn_moves = self.move_patterns.pawn(gamestate.active_color());
        let own_pieces = gamestate.board()[gamestate.active_color()].all_pieces();

        own_pawns.into_iter().flat_map(move |from| {
            let moves = pawn_moves.get_move(&from) & !own_pieces;
            moves.into_iter().map(move |to| {
                Move::new(from, to, Piece::Pawn) //, MoveNote::StandardMove)
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
                Move::new(from, to, Piece::Knight) //, MoveNote::StandardMove)
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
                Move::new(from, to, Piece::King) //, MoveNote::StandardMove)
            })
        })
    }
}
