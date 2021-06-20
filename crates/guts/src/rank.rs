use crate::color::Color;
use crate::FenParseError;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Rank::R1 => '1',
            Rank::R2 => '2',
            Rank::R3 => '3',
            Rank::R4 => '4',
            Rank::R5 => '5',
            Rank::R6 => '6',
            Rank::R7 => '7',
            Rank::R8 => '8',
        };

        write!(f, "{}", c)
    }
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

    pub const INNER: [Rank; 6] = [Rank::R2, Rank::R3, Rank::R4, Rank::R5, Rank::R6, Rank::R7];

    pub fn from_u8_panic(i: u8) -> Self {
        match i {
            0 => Rank::R1,
            1 => Rank::R2,
            2 => Rank::R3,
            3 => Rank::R4,
            4 => Rank::R5,
            5 => Rank::R6,
            6 => Rank::R7,
            7 => Rank::R8,
            _ => panic!("No rank for index {}", i),
        }
    }

    pub fn index(&self) -> usize {
        u8::from(*self) as usize
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            Rank::R1 => None,
            Rank::R2 => Some(Rank::R1),
            Rank::R3 => Some(Rank::R2),
            Rank::R4 => Some(Rank::R3),
            Rank::R5 => Some(Rank::R4),
            Rank::R6 => Some(Rank::R5),
            Rank::R7 => Some(Rank::R6),
            Rank::R8 => Some(Rank::R7),
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Rank::R1 => Some(Rank::R2),
            Rank::R2 => Some(Rank::R3),
            Rank::R3 => Some(Rank::R4),
            Rank::R4 => Some(Rank::R5),
            Rank::R5 => Some(Rank::R6),
            Rank::R6 => Some(Rank::R7),
            Rank::R7 => Some(Rank::R8),
            Rank::R8 => None,
        }
    }

    pub fn pawn_two_squares(color: Color) -> Self {
        match color {
            Color::White => Rank::R2,
            Color::Black => Rank::R7,
        }
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> Ordering {
        u8::from(*self).cmp(&u8::from(*other))
    }
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
    type Error = FenParseError;

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
            _ => Err(FenParseError::InvalidRank(c)),
        }
    }
}
