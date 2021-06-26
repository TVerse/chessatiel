use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Centipawn(pub i64);

impl Centipawn {
    pub const ZERO: Centipawn = Centipawn(0);

    pub const WIN: Centipawn = Centipawn(i64::MAX / 2);
    pub const MAX: Centipawn = Centipawn(i64::MAX);
    pub const LOSS: Centipawn = Centipawn(i64::MIN / 2);
    pub const MIN: Centipawn = Centipawn(i64::MIN + 1); // to avoid -MIN=MIN
}

impl PartialOrd for Centipawn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Centipawn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Neg for Centipawn {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
