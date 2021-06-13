mod make_move;

use crate::board::Board;
use crate::castling_rights::CastlingRights;
use crate::color::Color;
use crate::fen::RawFen;
use crate::parse_error::ParseError::InvalidHalfMoveClock;
use crate::square::Square;
use crate::ParseError;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position {
    board: Board,
    active_color: Color,
    castle_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u64,
    fullmove_number: u64,
}

impl Position {
    pub fn new(
        board: Board,
        active_color: Color,
        castle_rights: CastlingRights,
        en_passant: Option<Square>,
        halfmove_clock: u64,
        fullmove_number: u64,
    ) -> Self {
        Self {
            board,
            active_color,
            castle_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn active_color(&self) -> Color {
        self.active_color
    }

    pub fn castle_rights(&self) -> &CastlingRights {
        &self.castle_rights
    }

    pub fn en_passant(&self) -> &Option<Square> {
        &self.en_passant
    }
}

impl Position {
    fn parse_en_passant(s: &str) -> Result<Option<Square>, ParseError> {
        if s == "-" {
            Ok(None)
        } else {
            Square::from_str(s).map(Some)
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(
            Board::default(),
            Color::White,
            CastlingRights::default(),
            None,
            0,
            1,
        )
    }
}

impl FromStr for Position {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_fen = RawFen::parse(s)?;

        let active_color = Color::from_str(raw_fen.active_color)?;
        let castle_rights = CastlingRights::from_str(raw_fen.castling)?;
        let en_passant = Self::parse_en_passant(raw_fen.en_passant)?;
        let pieces = Board::from_str(raw_fen.pieces)?;
        let halfmove_clock = u64::from_str(raw_fen.halfmove_clock)
            .map_err(|_| InvalidHalfMoveClock(raw_fen.halfmove_clock.to_owned()))?;
        let fullmove_number = u64::from_str(raw_fen.fullmove_number)
            .map_err(|_| InvalidHalfMoveClock(raw_fen.halfmove_clock.to_owned()))?;

        Ok(Self::new(
            pieces,
            active_color,
            castle_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::PieceArray;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn parse_initial_board() {
        let initial_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let result = Position::from_str(initial_board).unwrap();

        assert_eq!(result, Position::default())
    }

    #[test]
    fn parse_1_e4() {
        let board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let expected = {
            use crate::color::Color::*;
            use crate::piece::Piece::*;
            let pieces = [
                [
                    Some((Rook, Black)),
                    Some((Knight, Black)),
                    Some((Bishop, Black)),
                    Some((Queen, Black)),
                    Some((King, Black)),
                    Some((Bishop, Black)),
                    Some((Knight, Black)),
                    Some((Rook, Black)),
                ],
                [
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                ],
                [None; 8],
                [None; 8],
                [
                    None,
                    None,
                    None,
                    None,
                    Some((Pawn, White)),
                    None,
                    None,
                    None,
                ],
                [None; 8],
                [
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    None,
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                ],
                [
                    Some((Rook, White)),
                    Some((Knight, White)),
                    Some((Bishop, White)),
                    Some((Queen, White)),
                    Some((King, White)),
                    Some((Bishop, White)),
                    Some((Knight, White)),
                    Some((Rook, White)),
                ],
            ];
            let pieces = PieceArray(pieces);
            Position::new(
                Board::from_piece_array(&pieces),
                Color::Black,
                CastlingRights::default(),
                Some(Square::new(File::E, Rank::R3)),
                0,
                1,
            )
        };

        let result = Position::from_str(board).unwrap();

        assert_eq!(result, expected)
    }

    #[test]
    fn initial_1_e4_different() {
        let initial_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let e4_board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let initial_result = Position::from_str(initial_board).unwrap();
        let e4_result = Position::from_str(e4_board).unwrap();

        assert_ne!(initial_result, e4_result)
    }

    #[test]
    fn clocks() {
        let board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 10 13";

        let expected = {
            use crate::color::Color::*;
            use crate::piece::Piece::*;
            let pieces = [
                [
                    Some((Rook, Black)),
                    Some((Knight, Black)),
                    Some((Bishop, Black)),
                    Some((Queen, Black)),
                    Some((King, Black)),
                    Some((Bishop, Black)),
                    Some((Knight, Black)),
                    Some((Rook, Black)),
                ],
                [
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                    Some((Pawn, Black)),
                ],
                [None; 8],
                [None; 8],
                [
                    None,
                    None,
                    None,
                    None,
                    Some((Pawn, White)),
                    None,
                    None,
                    None,
                ],
                [None; 8],
                [
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    None,
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                    Some((Pawn, White)),
                ],
                [
                    Some((Rook, White)),
                    Some((Knight, White)),
                    Some((Bishop, White)),
                    Some((Queen, White)),
                    Some((King, White)),
                    Some((Bishop, White)),
                    Some((Knight, White)),
                    Some((Rook, White)),
                ],
            ];
            let pieces = PieceArray(pieces);
            Position::new(
                Board::from_piece_array(&pieces),
                Color::Black,
                CastlingRights::default(),
                Some(Square::new(File::E, Rank::R3)),
                10,
                13,
            )
        };

        let result = Position::from_str(board).unwrap();

        assert_eq!(result, expected)
    }
}
