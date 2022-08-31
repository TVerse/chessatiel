use crate::Move;

pub trait MoveBuffer {
    fn push(&mut self, m: Move);
}

pub struct BasicMoveBuffer {
    inner: Vec<Move>,
}

impl MoveBuffer for BasicMoveBuffer {
    fn push(&mut self, m: Move) {
        self.inner.push(m)
    }
}

impl BasicMoveBuffer {
    pub fn new() -> Self {
        Self {
            inner: Vec::with_capacity(50),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for BasicMoveBuffer {
    fn default() -> Self {
        Self::new()
    }
}
