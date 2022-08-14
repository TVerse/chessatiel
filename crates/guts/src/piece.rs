use crate::FenParseError;
#[cfg(test)]
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        };

        write!(f, "{}", c)
    }
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

    pub const PROMOTION_TARGETS: [Piece; 4] =
        [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];

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

    pub fn from_usize_panic(idx: usize) -> Piece {
        match idx {
            0 => Piece::Pawn,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Rook,
            4 => Piece::Queen,
            5 => Piece::King,
            _ => panic!("Invalid idx for piece: {}", idx),
        }
    }

    pub fn is_slider(&self) -> bool {
        matches!(self, Piece::Bishop | Piece::Rook | Piece::Queen)
    }
}

impl TryFrom<char> for Piece {
    type Error = FenParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'K' => Ok(Piece::King),
            'Q' => Ok(Piece::Queen),
            'R' => Ok(Piece::Rook),
            'B' => Ok(Piece::Bishop),
            'N' => Ok(Piece::Knight),
            'P' => Ok(Piece::Pawn),
            _ => Err(FenParseError::InvalidPiece(c)),
        }
    }
}

#[cfg(test)]
impl PartialOrd for Piece {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for Piece {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index().cmp(&other.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_idx_should_be_invertible() {
        for p in Piece::ALL.iter() {
            assert_eq!(*p, Piece::from_usize_panic(p.index()))
        }
    }
}
