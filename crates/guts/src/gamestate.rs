use crate::board::Board;
use crate::castling_rights::CastlingRights;
use crate::color::Color;
use crate::fen::RawFen;
use crate::square::Square;
use crate::{Move, ParseError};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    castle_rights: CastlingRights,
    en_passant: Option<Square>,
}

impl Gamestate {
    pub fn new(
        board: Board,
        active_color: Color,
        castle_rights: CastlingRights,
        en_passant: Option<Square>,
    ) -> Self {
        Self {
            board,
            active_color,
            castle_rights,
            en_passant,
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

    pub fn make_move(&mut self, chess_move: &Move) {
        self.board.make_move(chess_move, self.active_color);
        self.active_color = !self.active_color;
    }
}

impl Gamestate {
    fn parse_en_passant(s: &str) -> Result<Option<Square>, ParseError> {
        if s == "-" {
            Ok(None)
        } else {
            Square::from_str(s).map(Some)
        }
    }
}

impl Default for Gamestate {
    fn default() -> Self {
        Self::new(
            Board::default(),
            Color::White,
            CastlingRights::default(),
            None,
        )
    }
}

impl FromStr for Gamestate {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_fen = RawFen::parse(s)?;

        let active_color = Color::from_str(raw_fen.active_color)?;
        let castle_rights = CastlingRights::from_str(raw_fen.castling)?;
        let en_passant = Self::parse_en_passant(raw_fen.en_passant)?;
        let pieces = Board::from_str(raw_fen.pieces)?;

        Ok(Self::new(pieces, active_color, castle_rights, en_passant))
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

        let result = Gamestate::from_str(initial_board).unwrap();

        assert_eq!(result, Gamestate::default())
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
            Gamestate::new(
                Board::from_piece_array(&pieces),
                Color::Black,
                CastlingRights::default(),
                Some(Square::new(File::E, Rank::R3)),
            )
        };

        let result = Gamestate::from_str(board).unwrap();

        assert_eq!(result, expected)
    }

    #[test]
    fn initial_1_e4_different() {
        let initial_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let e4_board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let initial_result = Gamestate::from_str(initial_board).unwrap();
        let e4_result = Gamestate::from_str(e4_board).unwrap();

        assert_ne!(initial_result, e4_result)
    }
}
