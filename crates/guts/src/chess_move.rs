use crate::piece::Piece;
use crate::square::Square;

// pub enum MoveNote {
//     StandardMove,
// }

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    // move_notes: MoveNote
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece) -> Self {
        Self { from, to, piece }
    }

    // pub fn new(from: Square, to: Square, piece: Piece, move_notes: MoveNote) -> Self {
    //     Self { from, to, piece, move_notes}
    // }
}
