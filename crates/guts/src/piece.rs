use crate::ParseError;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<char> for Piece {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'K' => Ok(Piece::King),
            'Q' => Ok(Piece::Queen),
            'R' => Ok(Piece::Rook),
            'B' => Ok(Piece::Bishop),
            'N' => Ok(Piece::Knight),
            'P' => Ok(Piece::Pawn),
            _ => Err(ParseError::InvalidPiece(c)),
        }
    }
}
