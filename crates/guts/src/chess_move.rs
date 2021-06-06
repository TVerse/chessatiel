use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use crate::file::File;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::Square;
use crate::ParseError;
use std::cmp::Ordering;

// pub enum MoveNote {
//     StandardMove,
// }

#[derive(Debug, Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub extra: Option<ExtraMoveInfo>,
    // move_notes: MoveNote
}

impl Move {
    pub fn new(from: Square, to: Square, extra: Option<ExtraMoveInfo>) -> Self {
        Self { from, to, extra }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}

impl FromStr for Move {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            Err(ParseError::InvalidMove(s.to_owned()))
        } else {
            let from = Square::from_str(&s[0..2])?;
            let to = Square::from_str(&s[2..4])?;
            Ok(Self::new(from, to, None))
        }
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for Move {}

#[cfg(test)]
impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.from.cmp(&other.from).then(self.to.cmp(&other.to))
    }
}

#[derive(Debug, Clone)]
pub struct ExtraMoveInfo {
    pub piece: Piece,
}

impl ExtraMoveInfo {
    pub fn new(piece: Piece) -> Self {
        Self { piece }
    }
}
