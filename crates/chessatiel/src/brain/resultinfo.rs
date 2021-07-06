use crate::brain::Centipawn;
use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Score {
    score: Centipawn,
    mate_depth: Option<isize>,
}

impl Score {
    pub fn new(score: Centipawn, mate_depth: Option<isize>) -> Self {
        Self { score, mate_depth }
    }

    pub fn score(&self) -> Centipawn {
        self.score
    }

    pub fn mate_depth(&self) -> Option<isize> {
        self.mate_depth
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then(match (self.mate_depth, other.mate_depth) {
                (Some(s), Some(o)) => s.cmp(&o),
                (Some(s), None) | (None, Some(s)) => s.cmp(&0),
                (None, None) => Ordering::Equal,
            })
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            score: -self.score,
            mate_depth: self.mate_depth.map(|i| -i),
        }
    }
}
