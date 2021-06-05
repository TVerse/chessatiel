use crate::ParseError;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const ALL: [Color; 2] = [Color::White, Color::Black];
}

impl FromStr for Color {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            Err(ParseError::InvalidColor(s.to_owned()))
        } else if s.contains('w') {
            Ok(Color::White)
        } else if s.contains('b') {
            Ok(Color::Black)
        } else {
            Err(ParseError::InvalidColor(s.to_owned()))
        }
    }
}
