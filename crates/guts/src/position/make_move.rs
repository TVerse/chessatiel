use crate::bitboard::Bitboard;
use crate::castling_rights::SinglePlayerCastlingRights;
use crate::chess_move::MoveType;
use crate::color::Color;
use crate::file::File;
use crate::position::UnmakeHistory;
use crate::rank::Rank;
use crate::square::Square;
use crate::{Move, Piece, Position};

// TODO unmake move
impl Position {
    // Assumes self and move are internally consistent
    // TODO debug asserts to verify internal consistency
    pub fn make_move(&mut self, chess_move: &Move) {
        let unmake_history = {
            let halfmove_clock = self.state.halfmove_clock;
            let castle_rights = self.castle_rights().clone();
            let en_passant = self.state.en_passant;
            move |captured| UnmakeHistory {
                halfmove_clock,
                castle_rights,
                en_passant,
                captured,
            }
        };
        let mut reset_half_move_clock = false;
        let mut en_passant = None;
        let mut captured = None;
        if chess_move.move_type().contains(MoveType::EN_PASSANT) {
            self.move_piece(chess_move.piece(), chess_move.from(), chess_move.to());
            let capture_square = Bitboard::from_square(chess_move.to())
                .forward_one(!self.state.active_color)
                .first_set_square()
                .unwrap();
            self.board[!self.state.active_color].clear_piece(Piece::Pawn, capture_square);
            self.hash
                .flip_piece(!self.state.active_color, Piece::Pawn, capture_square);
        } else if chess_move.move_type().contains(MoveType::CAPTURE) {
            captured = self.board[!self.state.active_color].piece_at(chess_move.to());
            self.board[!self.state.active_color].clear_all(chess_move.to());
            self.hash.flip_piece(
                !self.state.active_color,
                chess_move.piece(),
                chess_move.to(),
            );
            self.move_piece(chess_move.piece(), chess_move.from(), chess_move.to());
            reset_half_move_clock = true;
        } else if chess_move.move_type().contains(MoveType::PUSH) {
            self.move_piece(chess_move.piece(), chess_move.from(), chess_move.to());
            if (chess_move.piece() == Piece::Pawn)
                && (chess_move.to().rank() as i16 - chess_move.from().rank() as i16).abs() == 2
            {
                en_passant = Bitboard::from_square(chess_move.from())
                    .forward_one(self.state.active_color)
                    .first_set_square();
                self.hash.flip_ep_file(chess_move.from().file())
            }
        } else if chess_move
            .move_type()
            .intersects(MoveType::CASTLE_KINGSIDE | MoveType::CASTLE_QUEENSIDE)
        {
            let ((king_from, king_to), (rook_from, rook_to)) =
                if chess_move.move_type().contains(MoveType::CASTLE_KINGSIDE) {
                    kingside_castle_squares(self.state.active_color)
                } else {
                    queenside_castle_squares(self.state.active_color)
                };
            self.move_piece(Piece::King, king_from, king_to);
            self.move_piece(Piece::Rook, rook_from, rook_to);
        }

        if chess_move.piece() == Piece::King {
            if self.state.castle_rights[self.state.active_color].kingside {
                self.hash.flip_castle_rights(self.state.active_color, true);
            }
            if self.state.castle_rights[self.state.active_color].queenside {
                self.hash.flip_castle_rights(self.state.active_color, false);
            }
            self.state.castle_rights[self.state.active_color] = SinglePlayerCastlingRights::NONE;
        }

        if chess_move.piece() == Piece::Rook
            && chess_move.from().file() == File::A
            && self.state.castle_rights[self.state.active_color].queenside
        {
            self.state.castle_rights[self.state.active_color].queenside = false;
            self.hash.flip_castle_rights(self.state.active_color, false);
        } else if chess_move.piece() == Piece::Rook
            && chess_move.from().file() == File::H
            && self.state.castle_rights[self.state.active_color].kingside
        {
            self.state.castle_rights[self.state.active_color].kingside = false;
            self.hash.flip_castle_rights(self.state.active_color, true);
        }

        let opponent_queenside_castle_rook = {
            let (_, (sq, _)) = queenside_castle_squares(!self.state.active_color);
            Bitboard::from_square(sq)
        };
        let opponent_kingside_castle_rook = {
            let (_, (sq, _)) = kingside_castle_squares(!self.state.active_color);
            Bitboard::from_square(sq)
        };

        if Bitboard::from_square(chess_move.to()) & opponent_queenside_castle_rook
            != Bitboard::EMPTY
            && self.state.castle_rights[!self.state.active_color].queenside
        {
            self.state.castle_rights[!self.state.active_color].queenside = false;
            self.hash.flip_castle_rights(self.state.active_color, false)
        }
        if Bitboard::from_square(chess_move.to()) & opponent_kingside_castle_rook != Bitboard::EMPTY
            && self.state.castle_rights[!self.state.active_color].kingside
        {
            self.state.castle_rights[!self.state.active_color].kingside = false;
            self.hash.flip_castle_rights(self.state.active_color, true)
        }

        if chess_move.piece() == Piece::Pawn {
            reset_half_move_clock = true;
        }

        if let Some(p) = chess_move.promotion() {
            self.board[self.state.active_color].clear_piece(Piece::Pawn, chess_move.to());
            self.hash
                .flip_piece(self.state.active_color, Piece::Pawn, chess_move.to());
            self.board[self.state.active_color].set_piece(p, chess_move.to());
            self.hash
                .flip_piece(self.state.active_color, p, chess_move.to());
        }

        self.state.active_color = !self.state.active_color;
        self.hash.flip_side_to_move();
        if self.state.active_color == Color::White {
            self.state.fullmove_number += 1;
        }
        if reset_half_move_clock {
            self.state.halfmove_clock = 0;
        } else {
            self.state.halfmove_clock += 1;
        }
        if let Some(ep) = self.state.en_passant {
            self.hash.flip_ep_file(ep.file());
        }
        self.state.en_passant = en_passant;

        self.unmake_history.push(unmake_history(captured))
    }

    pub fn unmake_move(&mut self, chess_move: &Move) {
        let unmake_history = self.unmake_history.pop().unwrap();

        self.state.active_color = !self.state.active_color;
        self.hash.flip_side_to_move();

        if let Some(ep) = self.state.en_passant {
            self.hash.flip_ep_file(ep.file());
        }
        if let Some(ep) = unmake_history.en_passant {
            self.hash.flip_ep_file(ep.file());
        };
        self.state.en_passant = unmake_history.en_passant;

        if self.state.active_color == Color::Black {
            self.state.fullmove_number -= 1;
        }
        self.state.halfmove_clock = unmake_history.halfmove_clock;
        if let Some(p) = chess_move.promotion() {
            self.board[self.state.active_color].clear_piece(p, chess_move.to());
            self.hash
                .flip_piece(self.state.active_color, p, chess_move.to());
            self.board[self.state.active_color].set_piece(Piece::Pawn, chess_move.to());
            self.hash
                .flip_piece(self.state.active_color, Piece::Pawn, chess_move.to());
        }

        match (
            self.state.castle_rights[self.state.active_color].kingside,
            unmake_history.castle_rights[self.state.active_color].kingside,
        ) {
            (true, false) => self.hash.flip_castle_rights(self.state.active_color, true),
            (false, true) => self.hash.flip_castle_rights(self.state.active_color, true),
            (_, _) => {}
        }
        match (
            self.state.castle_rights[self.state.active_color].queenside,
            unmake_history.castle_rights[self.state.active_color].queenside,
        ) {
            (true, false) => self.hash.flip_castle_rights(self.state.active_color, false),
            (false, true) => self.hash.flip_castle_rights(self.state.active_color, false),
            (_, _) => {}
        }

        match (
            self.state.castle_rights[!self.state.active_color].kingside,
            unmake_history.castle_rights[!self.state.active_color].kingside,
        ) {
            (true, false) => self.hash.flip_castle_rights(!self.state.active_color, true),
            (false, true) => self.hash.flip_castle_rights(!self.state.active_color, true),
            (_, _) => {}
        }
        match (
            self.state.castle_rights[!self.state.active_color].queenside,
            unmake_history.castle_rights[!self.state.active_color].queenside,
        ) {
            (true, false) => self
                .hash
                .flip_castle_rights(!self.state.active_color, false),
            (false, true) => self
                .hash
                .flip_castle_rights(!self.state.active_color, false),
            (_, _) => {}
        }

        self.state.castle_rights = unmake_history.castle_rights;

        if chess_move.move_type().contains(MoveType::EN_PASSANT) {
            self.move_piece(chess_move.piece(), chess_move.to(), chess_move.from());
            let capture_square = Bitboard::from_square(chess_move.to())
                .forward_one(!self.state.active_color)
                .first_set_square()
                .unwrap();
            self.board[!self.state.active_color].set_piece(Piece::Pawn, capture_square);
            self.hash
                .flip_piece(!self.state.active_color, Piece::Pawn, capture_square);
        } else if chess_move.move_type().contains(MoveType::CAPTURE) {
            self.move_piece(chess_move.piece(), chess_move.to(), chess_move.from());
            self.board[!self.state.active_color]
                .set_piece(unmake_history.captured.unwrap(), chess_move.to());
            self.hash.flip_piece(
                !self.state.active_color,
                unmake_history.captured.unwrap(),
                chess_move.to(),
            );
        } else if chess_move.move_type().contains(MoveType::PUSH) {
            self.move_piece(chess_move.piece(), chess_move.to(), chess_move.from());
        } else if chess_move
            .move_type()
            .intersects(MoveType::CASTLE_KINGSIDE | MoveType::CASTLE_QUEENSIDE)
        {
            let ((king_from, king_to), (rook_from, rook_to)) =
                if chess_move.move_type().contains(MoveType::CASTLE_KINGSIDE) {
                    kingside_castle_squares(self.state.active_color)
                } else {
                    queenside_castle_squares(self.state.active_color)
                };
            self.move_piece(Piece::King, king_to, king_from);
            self.move_piece(Piece::Rook, rook_to, rook_from);
        }
    }

    fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.board[self.state.active_color].clear_piece(piece, from);
        self.hash.flip_piece(self.state.active_color, piece, from);
        self.board[self.state.active_color].set_piece(piece, to);
        self.hash.flip_piece(self.state.active_color, piece, to);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Move, Position};
    use std::str::FromStr;

    #[test]
    fn make_move_correct_result() {
        let mut pos =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let m = Move::new(
            Square::new(File::E, Rank::R2),
            Square::new(File::E, Rank::R4),
            Piece::Pawn,
            MoveType::PUSH,
            None,
        );
        pos.make_move(&m);

        let expected =
            Position::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
                .unwrap();

        assert_eq!(pos, expected)
    }

    #[test]
    fn make_move_correct_result_destroy_castle() {
        let mut pos = Position::from_str("8/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let m = Move::new(
            Square::new(File::H, Rank::R1),
            Square::new(File::G, Rank::R1),
            Piece::Rook,
            MoveType::PUSH,
            None,
        );
        pos.make_move(&m);

        let expected = Position::from_str("8/8/8/8/8/8/8/4K1R1 b - - 1 1").unwrap();

        assert_eq!(pos, expected)
    }

    #[test]
    fn make_move_correct_result_castle() {
        let mut pos = Position::from_str("8/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let m = Move::new(
            Square::new(File::E, Rank::R1),
            Square::new(File::G, Rank::R1),
            Piece::King,
            MoveType::CASTLE_KINGSIDE,
            None,
        );
        pos.make_move(&m);

        let expected = Position::from_str("8/8/8/8/8/8/8/5RK1 b - - 1 1").unwrap();

        assert_eq!(pos, expected)
    }

    #[test]
    fn make_then_unmake_move_is_identical_1() {
        let mut pos =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq a3 0 1")
                .unwrap();
        let orig_pos = pos.clone();
        let m = Move::new(
            Square::new(File::E, Rank::R2),
            Square::new(File::E, Rank::R3),
            Piece::Pawn,
            MoveType::PUSH,
            None,
        );
        pos.make_move(&m);
        pos.unmake_move(&m);

        assert_eq!(pos, orig_pos)
    }

    #[test]
    fn make_then_unmake_move_is_identical_2() {
        let mut pos =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq a3 0 1")
                .unwrap();
        let orig_pos = pos.clone();
        let m = Move::new(
            Square::new(File::E, Rank::R2),
            Square::new(File::E, Rank::R4),
            Piece::Pawn,
            MoveType::PUSH,
            None,
        );
        pos.make_move(&m);
        pos.unmake_move(&m);

        assert_eq!(pos, orig_pos)
    }
}
