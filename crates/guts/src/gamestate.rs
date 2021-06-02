use crate::castling_rights::CastlingRights;
use crate::color::Color;
use crate::fen::RawFen;
use crate::piece::Piece;
use crate::square::Square;
use crate::ParseError;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

type PieceArray = [[Option<(Piece, Color)>; 8]; 8];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GameState {
    pieces: PieceArray,
    active_color: Color,
    castle_rights: CastlingRights,
    en_passant: Option<Square>,
}

impl GameState {
    pub fn new(
        pieces: PieceArray,
        active_color: Color,
        castle_rights: CastlingRights,
        en_passant: Option<Square>,
    ) -> Self {
        Self {
            pieces,
            active_color,
            castle_rights,
            en_passant,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        use Color::*;
        use Piece::*;
        let pieces = [
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
            [Some((Pawn, White)); 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [Some((Pawn, Black)); 8],
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
        ];
        Self::new(pieces, Color::White, CastlingRights::default(), None)
    }
}

impl FromStr for GameState {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_fen = RawFen::parse(s)?;

        let active_color = Color::from_str(raw_fen.active_color)?;
        let castle_rights = CastlingRights::from_str(raw_fen.castling)?;
        let en_passant = parse_en_passant(raw_fen.en_passant)?;
        let pieces:PieceArray = parse_pieces(raw_fen.pieces)?;

        Ok(Self::new(pieces, active_color, castle_rights, en_passant))
    }
}

fn parse_en_passant(s: &str) -> Result<Option<Square>, ParseError> {
    if s == "-" {
        Ok(None)
    } else {
        Square::from_str(s).map(Some)
    }
}

fn parse_pieces(s: &str) -> Result<PieceArray, ParseError> {
    let ranks = s.split('/');
    let ranks: Vec<_> = ranks.collect();
    let len = ranks.len();
    let ranks: [&str; 8] = ranks
        .try_into()
        .map_err(|_| ParseError::WrongNumberOfRanks(len))?;

    let mut pieces: PieceArray = [[None; 8]; 8];
    for i in 0..ranks.len() {
        let rank = ranks[ranks.len() - i - 1];
        let chars = rank.chars();
        let mut ps: [Option<(Piece, Color)>; 8] = [None; 8];
        let mut idx: usize = 0;
        for c in chars {
            if let Some(n) = c.to_digit(10) {
                idx += n as usize;
            } else {
                let piece = Piece::try_from(c.to_ascii_uppercase())?;
                let color = if c.is_ascii_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };
                ps[idx] = Some((piece, color));
                idx += 1;
            }
        }
        if idx != 8 {
            return Err(ParseError::WrongNumberOfFiles(idx));
        }
        pieces[i] = ps;
    }

    Ok(pieces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn parse_initial_board() {
        let initial_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let result = GameState::from_str(initial_board).unwrap();

        assert_eq!(result, GameState::default())
    }

    #[test]
    fn parse_1_e4() {
        let board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let expected = {
            use Color::*;
            use Piece::*;
            let pieces = [
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
                [None; 8],
                [Some((Pawn, Black)); 8],
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
            ];
            GameState::new(
                pieces,
                Color::Black,
                CastlingRights::default(),
                Some(Square::new(File::E, Rank::R3)),
            )
        };

        let result = GameState::from_str(board).unwrap();

        assert_eq!(result, expected)
    }
}
