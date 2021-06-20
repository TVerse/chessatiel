use crate::bitboard::Bitboard;
use crate::castling_rights::SinglePlayerCastlingRights;
use crate::chess_move::MoveType;
use crate::color::Color;
use crate::file::File;
use crate::rank::Rank;
use crate::square::Square;
use crate::{Move, Piece, Position};

// TODO unmake move
impl Position {
    // Assumes self and move are internally consistent
    // TODO debug asserts to verify internal consistency
    pub fn make_move(&mut self, chess_move: &Move) {
        let mut reset_half_move_clock = false;
        let mut en_passant = None;
        if chess_move.move_type.contains(MoveType::EN_PASSANT) {
            self.move_piece(chess_move.piece, chess_move.from, chess_move.to);
            let pawn_square = Bitboard::from_square(chess_move.to)
                .forward_one(!self.active_color)
                .first_set_square()
                .unwrap();
            self.board[!self.active_color].clear_piece(Piece::Pawn, pawn_square);
        } else if chess_move.move_type.contains(MoveType::CAPTURE) {
            self.board[!self.active_color].clear_all(chess_move.to);
            self.move_piece(chess_move.piece, chess_move.from, chess_move.to);
            reset_half_move_clock = true;
        } else if chess_move.move_type.contains(MoveType::PUSH) {
            self.move_piece(chess_move.piece, chess_move.from, chess_move.to);
            if (chess_move.piece == Piece::Pawn)
                && (chess_move.to.rank() as i16 - chess_move.from.rank() as i16).abs() == 2
            {
                en_passant = Bitboard::from_square(chess_move.from)
                    .forward_one(self.active_color)
                    .first_set_square();
            }
        } else if chess_move
            .move_type
            .intersects(MoveType::CASTLE_KINGISDE | MoveType::CASTLE_QUEENSIDE)
        {
            let ((king_from, king_to), (rook_from, rook_to)) =
                if chess_move.move_type.contains(MoveType::CASTLE_KINGISDE) {
                    kingside_castle_squares(self.active_color)
                } else {
                    queenside_castle_squares(self.active_color)
                };
            self.move_piece(Piece::King, king_from, king_to);
            self.move_piece(Piece::Rook, rook_from, rook_to);
        }

        if chess_move.piece == Piece::King {
            self.castle_rights[self.active_color] = SinglePlayerCastlingRights::NONE;
        }

        if chess_move.piece == Piece::Rook && chess_move.from.file() == File::A {
            self.castle_rights[self.active_color].queenside = false;
        } else if chess_move.piece == Piece::Rook && chess_move.from.file() == File::H {
            self.castle_rights[self.active_color].kingside = false;
        }

        let opponent_kingside_castle_rook = {
            let (_, (sq, _)) = kingside_castle_squares(!self.active_color);
            Bitboard::from_square(sq)
        };

        let opponent_queenside_castle_rook = {
            let (_, (sq, _)) = queenside_castle_squares(!self.active_color);
            Bitboard::from_square(sq)
        };

        if Bitboard::from_square(chess_move.to) & opponent_kingside_castle_rook != Bitboard::EMPTY {
            self.castle_rights[!self.active_color].kingside = false;
        }
        if Bitboard::from_square(chess_move.to) & opponent_queenside_castle_rook != Bitboard::EMPTY
        {
            self.castle_rights[!self.active_color].queenside = false;
        }

        if chess_move.piece == Piece::Pawn {
            reset_half_move_clock = true;
        }

        if let Some(p) = chess_move.promotion {
            self.board[self.active_color].clear_piece(Piece::Pawn, chess_move.to);
            self.board[self.active_color].set_piece(p, chess_move.to);
        }

        self.active_color = !self.active_color;
        if self.active_color == Color::White {
            self.fullmove_number += 1;
        }
        if reset_half_move_clock {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        self.en_passant = en_passant;
    }

    fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.board[self.active_color].clear_all(from);
        self.board[self.active_color].set_piece(piece, to);
    }
}

fn kingside_castle_squares(color: Color) -> ((Square, Square), (Square, Square)) {
    match color {
        Color::White => (
            (
                Square::new(File::E, Rank::R1),
                Square::new(File::G, Rank::R1),
            ),
            (
                Square::new(File::H, Rank::R1),
                Square::new(File::F, Rank::R1),
            ),
        ),
        Color::Black => (
            (
                Square::new(File::E, Rank::R8),
                Square::new(File::G, Rank::R8),
            ),
            (
                Square::new(File::H, Rank::R8),
                Square::new(File::F, Rank::R8),
            ),
        ),
    }
}

fn queenside_castle_squares(color: Color) -> ((Square, Square), (Square, Square)) {
    match color {
        Color::White => (
            (
                Square::new(File::E, Rank::R1),
                Square::new(File::C, Rank::R1),
            ),
            (
                Square::new(File::A, Rank::R1),
                Square::new(File::D, Rank::R1),
            ),
        ),
        Color::Black => (
            (
                Square::new(File::E, Rank::R8),
                Square::new(File::C, Rank::R8),
            ),
            (
                Square::new(File::A, Rank::R8),
                Square::new(File::D, Rank::R8),
            ),
        ),
    }
}
