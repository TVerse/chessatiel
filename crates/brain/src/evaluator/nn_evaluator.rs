use crate::evaluator::Evaluator;
use crate::neural_networks::{Input, TwoHiddenLayerNetwork};
use crate::{CentipawnScore, SHARED_COMPONENTS};
use guts::{Color, Position};

pub struct NeuralNetworkEvaluator<'a> {
    nn: &'a TwoHiddenLayerNetwork<768, 64, 16, 1>,
}

impl NeuralNetworkEvaluator<'static> {
    pub fn new() -> Self {
        Self {
            nn: &SHARED_COMPONENTS.nn,
        }
    }
}

impl Default for NeuralNetworkEvaluator<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Evaluator for NeuralNetworkEvaluator<'a> {
    fn evaluate(&self, position: &Position) -> CentipawnScore {
        let input = Input::from_position(position);
        let output = self.nn.apply(&input)[0];
        let scaled_output = inverse_sigmoid(output) * 1000.0;
        let score_white = clip(scaled_output);
        if position.active_color() == Color::White {
            CentipawnScore(score_white)
        } else {
            CentipawnScore(-score_white)
        }
    }
}

fn inverse_sigmoid(a: f64) -> f64 {
    -f64::ln(2.0 / (a + 1.0) - 1.0)
}

fn clip(a: f64) -> i32 {
    if a > 16000.0 {
        16000
    } else if a < -16000.0 {
        -16000
    } else {
        a as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn relative_score() {
        let position = Position::default();
        let evaluator = NeuralNetworkEvaluator::default();
        let score_white = evaluator.evaluate(&position);
        let pos_black =
            Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        let score_black = evaluator.evaluate(&pos_black);
        assert_eq!(score_white, -score_black);
    }

    #[test]
    fn score() {
        let position = Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/8/4K3 w kq - 0 1").unwrap();
        let evaluator = NeuralNetworkEvaluator::default();
        panic!("{:?}", evaluator.evaluate(&position))
    }
}
