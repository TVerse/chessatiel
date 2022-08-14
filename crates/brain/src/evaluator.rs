use guts::Position;
use std::marker::PhantomData;
use std::ops::Neg;

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

#[derive(Default)]
pub struct PieceCountEvaluator {
    // Prevent construction
    _p: PhantomData<usize>,
}

impl PieceCountEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluator for PieceCountEvaluator {
    fn evaluate(&self, position: &Position) -> CentipawnScore {
        let my_pieces = position.board()[position.active_color()]
            .all_pieces()
            .count_ones() as i32;
        let their_pieces = position.board()[!position.active_color()]
            .all_pieces()
            .count_ones() as i32;

        CentipawnScore((my_pieces - their_pieces) * 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn piece_count_evaluator() {
        let position = Position::default();
        assert_eq!(
            PieceCountEvaluator::new().evaluate(&position),
            CentipawnScore::ZERO
        )
    }

    #[test]
    fn piece_count_evaluator_2() {
        let evaluator = PieceCountEvaluator::new();

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(-100));

        let position =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(evaluator.evaluate(&position), CentipawnScore(100));
    }
}
