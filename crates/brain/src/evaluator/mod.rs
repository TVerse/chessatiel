pub mod pst_evaluator;
#[cfg(test)]
mod test_evaluator;

use guts::Position;
use std::ops::Neg;

pub use pst_evaluator::PieceSquareTableEvaluator;
#[cfg(test)]
pub use test_evaluator::PieceCountEvaluator;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct CentipawnScore(pub i32);

impl CentipawnScore {
    pub const ZERO: Self = Self(0);
    pub const CHECKMATED: Self = Self(Self::MIN.0 / 2);
    pub const MAX: Self = Self(i32::MAX);
    pub const MIN: Self = Self(i32::MIN + 1); // To avoid -MIN = MIN
}

impl Neg for CentipawnScore {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

pub trait Evaluator {
    fn evaluate(&self, position: &Position) -> CentipawnScore;
}

#[derive(Debug, Copy, Clone)]
pub enum ScoreBound {
    Exact,
    Upper,
    Lower,
}
