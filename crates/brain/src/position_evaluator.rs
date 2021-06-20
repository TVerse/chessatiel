use crate::Centipawn;
use guts::Position;

pub struct PositionEvaluator {}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, position: &Position) -> Centipawn {
        let score = (position.board()[position.active_color()]
            .all_pieces()
            .count_ones() as f64)
            - (position.board()[!position.active_color()]
                .all_pieces()
                .count_ones() as f64);
        Centipawn(score)
    }
}

impl Default for PositionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn starting_position() {
        let evaluator = PositionEvaluator::new();
        let position = Position::default();

        assert_eq!(evaluator.evaluate(&position), Centipawn(0.0))
    }

    #[test]
    fn more_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("k7/8/8/8/8/8/8/KQ6 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Centipawn(1.0))
    }

    #[test]
    fn less_for_active() {
        let evaluator = PositionEvaluator::new();
        let position = Position::from_str("kq6/8/8/8/8/8/8/K7 w - - 0 1").unwrap();

        assert_eq!(evaluator.evaluate(&position), Centipawn(-1.0))
    }
}
