use crate::{Move, MoveType};
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct PriorityMove {
    m: Move,
    p: u8,
}

impl PartialOrd<Self> for PriorityMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.p.cmp(&other.p)
    }
}

#[derive(Debug)]
pub struct MoveBuffer {
    inner: Vec<PriorityMove>,
}

impl MoveBuffer {
    pub fn new() -> Self {
        Self {
            inner: Vec::with_capacity(50),
        }
    }

    pub fn push(&mut self, m: Move) {
        self.inner.push(PriorityMove { p: default_priority(&m), m })
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn set_priority(&mut self, m: &Move, priority: u8) -> bool {
        self.inner.iter_mut().find(|pm| &pm.m == m).map(|pm| pm.p = priority).is_some()
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.find_highest();
        self.inner.pop().map(|pm| pm.m)
    }

    fn find_highest(&mut self) {
        let len = self.inner.len();
        if len == 0 {
            return;
        }
        let mut highest_idx = 0;
        let mut highest_p = u8::MIN;
        for i in 0..len {
            let p = self.inner[i].p;
            if p > highest_p {
                highest_idx = i;
                highest_p = p;
                if p == u8::MAX {
                    break;
                }
            }
        }
        self.inner.swap(len - 1, highest_idx)
    }

    pub fn unordered_iter(&self) -> impl Iterator<Item=&Move> {
        self.inner.iter().map(|pm| &pm.m)
    }
}

fn default_priority(m: &Move) -> u8 {
    if m.move_type().contains(MoveType::CAPTURE) {
        10
    } else if m.promotion().is_some() {
        9
    } else {
        u8::MIN
    }
}

impl Default for MoveBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{MoveType, Piece, Square};
    use super::*;

    #[test]
    fn highest_prio_first() {
        let m0 = Move::new(Square::from_index(0), Square::from_index(63), Piece::King, MoveType::PUSH, None);
        let m1 = Move::new(Square::from_index(1), Square::from_index(63), Piece::King, MoveType::PUSH, None);
        let mut buf = MoveBuffer::new();
        buf.push(m0.clone());
        buf.push(m1.clone());
        buf.set_priority(&m0, 10);
        buf.set_priority(&m1, 9);
        assert_eq!(buf.pop(), Some(m0.clone()));
        buf.push(m1.clone());
        buf.push(m0.clone());
        buf.set_priority(&m0, 10);
        buf.set_priority(&m1, 9);
        assert_eq!(buf.pop(), Some(m0))
    }
}