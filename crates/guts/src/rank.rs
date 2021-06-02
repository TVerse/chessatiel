use crate::ParseError;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

impl Rank {
    pub const ALL: [Rank; 8] = [
        Rank::R1,
        Rank::R2,
        Rank::R3,
        Rank::R4,
        Rank::R5,
        Rank::R6,
        Rank::R7,
        Rank::R8,
    ];
}

impl From<Rank> for u8 {
    fn from(r: Rank) -> Self {
        match r {
            Rank::R1 => 0,
            Rank::R2 => 1,
            Rank::R3 => 2,
            Rank::R4 => 3,
            Rank::R5 => 4,
            Rank::R6 => 5,
            Rank::R7 => 6,
            Rank::R8 => 7,
        }
    }
}

impl TryFrom<char> for Rank {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '1' => Ok(Rank::R1),
            '2' => Ok(Rank::R2),
            '3' => Ok(Rank::R3),
            '4' => Ok(Rank::R4),
            '5' => Ok(Rank::R5),
            '6' => Ok(Rank::R6),
            '7' => Ok(Rank::R7),
            '8' => Ok(Rank::R8),
            _ => Err(ParseError::InvalidRank(c)),
        }
    }
}
