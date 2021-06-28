use crate::FenParseError;
use std::fmt;
use std::ops::Not;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const NUM: usize = Self::ALL.len();

    pub const ALL: [Self; 2] = [Self::White, Self::Black];

    pub fn index(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl FromStr for Color {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(FenParseError::InvalidColor(s.to_owned()))
        } else if s.contains('w') {
            Ok(Color::White)
        } else if s.contains('b') {
            Ok(Color::Black)
        } else {
            Err(FenParseError::InvalidColor(s.to_owned()))
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::White => "w",
                Color::Black => "b",
            }
        )
    }
}

impl From<Color> for usize {
    fn from(c: Color) -> Self {
        match c {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}
