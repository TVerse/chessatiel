use crate::color::Color;
use crate::ParseError;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CastlingRights {
    white: SinglePlayerCastlingRights,
    black: SinglePlayerCastlingRights,
}

impl CastlingRights {
    pub fn new(white: SinglePlayerCastlingRights, black: SinglePlayerCastlingRights) -> Self {
        Self { white, black }
    }
}

impl Index<Color> for CastlingRights {
    type Output = SinglePlayerCastlingRights;

    fn index(&self, index: Color) -> &Self::Output {
        match index {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}

impl IndexMut<Color> for CastlingRights {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        match index {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SinglePlayerCastlingRights {
    pub kingside: bool,
    pub queenside: bool,
}

impl SinglePlayerCastlingRights {
    pub const NONE: Self = Self {
        kingside: false,
        queenside: false,
    };

    pub fn new(kingside: bool, queenside: bool) -> SinglePlayerCastlingRights {
        Self {
            kingside,
            queenside,
        }
    }
}

impl Default for SinglePlayerCastlingRights {
    fn default() -> Self {
        SinglePlayerCastlingRights::new(true, true)
    }
}

impl FromStr for CastlingRights {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let white_kingside = s.contains('K');
        let white_queenside = s.contains('Q');
        let black_kingside = s.contains('k');
        let black_queenside = s.contains('q');

        let white = SinglePlayerCastlingRights::new(white_kingside, white_queenside);
        let black = SinglePlayerCastlingRights::new(black_kingside, black_queenside);

        Ok(CastlingRights::new(white, black))
    }
}
