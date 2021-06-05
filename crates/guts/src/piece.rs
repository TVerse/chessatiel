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

impl Piece {
    pub const NUM: usize = 6;

    pub const ALL: [Piece; Self::NUM] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    pub fn index(&self) -> usize {
        match self {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }
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
