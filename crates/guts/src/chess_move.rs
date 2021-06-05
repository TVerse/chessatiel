use crate::square::Square;

pub struct Move {
    from: Square,
    to: Square,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}
