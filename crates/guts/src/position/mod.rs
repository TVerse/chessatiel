mod make_move;
pub mod zobrist;

use crate::board::Board;
use crate::castling_rights::CastlingRights;
use crate::color::Color;
use crate::fen::RawFen;
use crate::parse_error::FenParseError::InvalidHalfMoveClock;
use crate::position::zobrist::{Zobrist, ZobristHash};
use crate::square::Square;
use crate::FenParseError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct State {
    active_color: Color,
    castle_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u64,
    fullmove_number: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position {
    board: Board,
    state: State,

    hash: ZobristHash,
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
        let state = State {
            active_color,
            castle_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        };
        let hash = Zobrist::get().for_position(&board, &state);
        Self { board, state, hash }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn active_color(&self) -> Color {
        self.state.active_color
    }

    pub fn castle_rights(&self) -> &CastlingRights {
        &self.state.castle_rights
    }

    pub fn castle_rights_mut(&mut self) -> &mut CastlingRights {
        &mut self.state.castle_rights
    }

    pub fn en_passant(&self) -> &Option<Square> {
        &self.state.en_passant
    }

    pub fn halfmove_clock(&self) -> u64 {
        self.state.halfmove_clock
    }

    pub fn fullmove_number(&self) -> u64 {
        self.state.fullmove_number
    }

    pub fn hash(&self) -> ZobristHash {
        self.hash
    }
}

impl Position {
    fn parse_en_passant(s: &str) -> Result<Option<Square>, FenParseError> {
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
    type Err = FenParseError;

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

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let en_passant = match self.en_passant() {
            Some(sq) => sq.to_string(),
            None => "-".to_string(),
        };
        write!(
            f,
            "{} {} {} {} {} {}",
            self.board(),
            self.active_color(),
            self.castle_rights(),
            en_passant,
            self.halfmove_clock(),
            self.fullmove_number()
        )
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

    #[test]
    fn fen_both_ways() {
        let initial_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let e4_board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let kiwipete = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let kiwipete_no_king_castle =
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w Qq - 0 1";

        assert_eq!(
            Position::from_str(initial_board).unwrap().to_string(),
            initial_board
        );
        assert_eq!(Position::from_str(e4_board).unwrap().to_string(), e4_board);
        assert_eq!(Position::from_str(kiwipete).unwrap().to_string(), kiwipete);
        assert_eq!(
            Position::from_str(kiwipete_no_king_castle)
                .unwrap()
                .to_string(),
            kiwipete_no_king_castle
        );
    }

    #[test]
    fn zobrist_startpos_not_zero() {
        let startpos = Position::default();

        assert_ne!(startpos.hash, ZobristHash(0))
    }
}
