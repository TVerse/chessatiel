#[cfg(test)]
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::piece::Piece;
use crate::square::Square;
use crate::ParseError;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Ord, PartialOrd))]
pub struct MoveCore {
    pub from: Square,
    pub to: Square,
}

impl MoveCore {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}

impl fmt::Display for MoveCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}

impl FromStr for MoveCore {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            Err(ParseError::InvalidMove(s.to_owned()))
        } else {
            let from = Square::from_str(&s[0..2])?;
            let to = Square::from_str(&s[2..4])?;
            Ok(Self::new(from, to))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoveType {
    Push,
    Capture,
}

impl fmt::Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
impl PartialOrd for MoveType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for MoveType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Push, Self::Capture) => Ordering::Greater,
            (Self::Capture, Self::Push) => Ordering::Less,
            (_, _) => Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Ord, PartialOrd))]
pub struct Move {
    pub core_move: MoveCore,
    pub piece: Piece,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece, move_type: MoveType) -> Self {
        Self {
            core_move: MoveCore::new(from, to),
            piece,
            move_type,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.core_move, self.move_type)
    }
}
