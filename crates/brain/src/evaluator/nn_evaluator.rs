use crate::evaluator::Evaluator;
use crate::neural_networks::{Input, Network};
use crate::{CentipawnScore, SHARED_COMPONENTS};
use guts::Position;

pub struct NeuralNetworkEvaluator<'a> {
    nn: &'a Network,
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
        let input = Input::new(position);
        let output = self.nn.apply(&input);
        let scaled_output = inverse_sigmoid(output) * 410.0; // Scaling factor from Stockfish/nnue-pytorch
        CentipawnScore(clip(scaled_output))
    }
}

fn inverse_sigmoid(a: f64) -> f64 {
    -f64::ln(1.0 / a - 1.0)
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
