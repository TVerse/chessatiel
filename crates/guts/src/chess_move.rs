#[cfg(test)]
use std::cmp::Ordering;
use std::fmt;

use crate::piece::Piece;
use crate::square::Square;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MoveType: u8 {
        const PUSH = 0b00000001;
        const CAPTURE = 0b00000010;
        const EN_PASSANT = 0b00000100;
        const CASTLE_KINGSIDE = 0b00001000;
        const CASTLE_QUEENSIDE = 0b00010000;
    }
}

impl fmt::Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    piece: Piece,
    move_type: MoveType,
    promotion: Option<Piece>,
}

impl Move {
    pub fn new(
        from: Square,
        to: Square,
        piece: Piece,
        move_type: MoveType,
        promotion: Option<Piece>,
    ) -> Self {
        Self {
            from,
            to,
            piece,
            move_type,
            promotion,
        }
    }

    pub fn as_uci(&self) -> String {
        let promotion_str = self
            .promotion
            .map(|p| p.to_string().to_ascii_lowercase())
            .unwrap_or_else(|| "".to_string());
        format!("{}{}{}", self.from, self.to, promotion_str)
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn move_type(&self) -> MoveType {
        self.move_type
    }

    pub fn promotion(&self) -> Option<Piece> {
        self.promotion
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.as_uci(), self.move_type)
    }
}

#[cfg(test)]
impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.from.cmp(&other.from).then(
            self.to
                .cmp(&other.to)
                .then(self.promotion.cmp(&other.promotion)),
        )
    }
}
