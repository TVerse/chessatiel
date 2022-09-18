pub mod heap_arrays;

use crate::neural_networks::heap_arrays::{HeapArray, HeapMatrix};
use guts::{Color, Piece, Position, Square};
use rand::{Rng, RngCore};

pub struct Input {
    inner: HeapArray<f64, 768>,
}

impl Input {
    pub fn new(pos: &Position) -> Self {
        let mut inner = HeapArray::new(vec![0.0; 768]);

        for s in Square::ALL {
            for p in Piece::ALL {
                for c in Color::ALL {
                    let v = if pos.board()[c][p].is_set(s) {
                        1.0
                    } else {
                        0.0
                    };
                    inner[Self::index_for(s, p, c)] = v
                }
            }
        }

        Self { inner }
    }

    pub fn from_array(inner: HeapArray<f64, 768>) -> Self {
        Self { inner }
    }

    fn index_for(s: Square, p: Piece, c: Color) -> usize {
        s.bitboard_index() + 64 * p.index() + 64 * 6 * c.index()
    }
}

pub trait Layer<T, const INPUTS: usize, const NEURONS: usize> {
    fn apply(&self, input: &HeapArray<T, INPUTS>) -> HeapArray<T, NEURONS>;
}

pub trait TrainableLayer<T, const INPUTS: usize, const NEURONS: usize> {
    fn apply(&mut self, input: &HeapArray<T, INPUTS>) -> HeapArray<T, NEURONS>;

    fn average_from_count(&mut self, count: usize);

    fn activations(&self) -> &HeapArray<T, NEURONS>;

    fn derivatives(&self) -> &HeapArray<T, NEURONS>;

    fn clear(&mut self);
}

#[derive(Debug, Clone)]
pub struct FullyConnectedLayer<const INPUTS: usize, const NEURONS: usize> {
    input_weights: HeapMatrix<f64, NEURONS, INPUTS>,
    bias_weights: HeapArray<f64, NEURONS>,
    activation_function: ActivationFunction,
}

impl<const INPUTS: usize, const NEURONS: usize> FullyConnectedLayer<INPUTS, NEURONS> {
    pub fn zeroed(activation_function: ActivationFunction) -> Self {
        Self {
            input_weights: HeapMatrix::zeroed(),
            bias_weights: HeapArray::zeroed(),
            activation_function,
        }
    }

    pub fn random(rng: &mut dyn RngCore, activation_function: ActivationFunction) -> Self {
        let mut input_weights = HeapMatrix::zeroed();
        let mut bias_weights = HeapArray::zeroed();
        for i in 0..input_weights.len() {
            rng.fill(&mut input_weights[i])
        }
        rng.fill(&mut bias_weights);
        Self {
            input_weights,
            bias_weights,
            activation_function,
        }
    }
}

impl<const INPUTS: usize, const NEURONS: usize> Layer<f64, INPUTS, NEURONS>
    for FullyConnectedLayer<INPUTS, NEURONS>
{
    fn apply(&self, input: &HeapArray<f64, INPUTS>) -> HeapArray<f64, NEURONS> {
        let mut out = HeapArray::zeroed();
        for i in 0..NEURONS {
            let weights = &self.input_weights[i];
            let bias = self.bias_weights[i];
            out[i] = (self.activation_function.activation_fn)(input.dot(weights) + bias)
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct TrainableFullyConnectedLayer<const INPUTS: usize, const NEURONS: usize> {
    fcl: FullyConnectedLayer<INPUTS, NEURONS>,
    activations: HeapArray<f64, NEURONS>,
    derivatives: HeapArray<f64, NEURONS>,
}

impl<const INPUTS: usize, const NEURONS: usize> TrainableFullyConnectedLayer<INPUTS, NEURONS> {
    pub fn new(fcl: FullyConnectedLayer<INPUTS, NEURONS>) -> Self {
        Self {
            fcl,
            activations: HeapArray::zeroed(),
            derivatives: HeapArray::zeroed(),
        }
    }
}

impl<const INPUTS: usize, const NEURONS: usize> TrainableLayer<f64, INPUTS, NEURONS>
    for TrainableFullyConnectedLayer<INPUTS, NEURONS>
{
    fn apply(&mut self, input: &HeapArray<f64, INPUTS>) -> HeapArray<f64, NEURONS> {
        let mut out = HeapArray::zeroed();
        let mut derivatives = HeapArray::zeroed();
        for i in 0..NEURONS {
            let weights = &self.fcl.input_weights[i];
            let bias = self.fcl.bias_weights[i];
            let z = input.dot(weights) + bias;
            out[i] = (self.fcl.activation_function.activation_fn)(z);
            derivatives[i] = (self.fcl.activation_function.derivative)(z);
        }
        // TODO batches
        self.activations = out.clone();
        self.derivatives = derivatives;

        out
    }

    fn average_from_count(&mut self, count: usize) {
        for i in 0..NEURONS {
            self.activations[i] /= count as f64;
            self.derivatives[i] /= count as f64;
        }
    }

    fn activations(&self) -> &HeapArray<f64, NEURONS> {
        &self.activations
    }

    fn derivatives(&self) -> &HeapArray<f64, NEURONS> {
        &self.derivatives
    }

    fn clear(&mut self) {
        self.activations = HeapArray::zeroed();
        self.derivatives = HeapArray::zeroed();
    }
}

pub fn error_function<const N: usize>(
    output: &HeapArray<f64, N>,
    expected: &HeapArray<f64, N>,
) -> f64 {
    (output - expected).squared_size() / (2.0 * N as f64)
}

pub fn error_derivative<const N: usize>(
    output: &HeapArray<f64, N>,
    expected: &HeapArray<f64, N>,
) -> f64 {
    (output - expected).sum()
}

#[derive(Debug, Copy, Clone)]
pub struct ActivationFunction {
    activation_fn: fn(f64) -> f64,
    derivative: fn(f64) -> f64,
}

impl ActivationFunction {
    pub const CLIPPED_RELU: ActivationFunction = ActivationFunction {
        activation_fn: activation_functions::clipped_relu,
        derivative: activation_functions::clipped_relu_derivative,
    };

    pub const SIGMOID: ActivationFunction = ActivationFunction {
        activation_fn: activation_functions::sigmoid,
        derivative: activation_functions::sigmoid_derivative,
    };

    pub const SCALED_TRANSLATED_SIGMOID: ActivationFunction = ActivationFunction {
        activation_fn: activation_functions::scaled_translated_sigmoid,
        derivative: activation_functions::scaled_translated_sigmoid_derivative,
    };

    pub const RELU: ActivationFunction = ActivationFunction {
        activation_fn: activation_functions::relu,
        derivative: activation_functions::relu_derivative,
    };

    pub fn new(activation_fn: fn(f64) -> f64, derivative: fn(f64) -> f64) -> Self {
        Self {
            activation_fn,
            derivative,
        }
    }

    pub fn activation_at(&self, a: f64) -> f64 {
        (self.activation_fn)(a)
    }

    pub fn derivative_at(&self, a: f64) -> f64 {
        (self.derivative)(a)
    }
}

mod activation_functions {
    pub fn clipped_relu(a: f64) -> f64 {
        a.clamp(0.0, 1.0)
    }

    pub fn clipped_relu_derivative(a: f64) -> f64 {
        if (0.0..1.0).contains(&a) {
            1.0
        } else {
            0.0
        }
    }

    pub fn relu(a: f64) -> f64 {
        a.max(0.0)
    }

    pub fn relu_derivative(a: f64) -> f64 {
        if a < 0.0 {
            0.0
        } else {
            1.0
        }
    }

    pub fn sigmoid(a: f64) -> f64 {
        1.0 / (1.0 + (-a).exp())
    }

    pub fn sigmoid_derivative(a: f64) -> f64 {
        sigmoid(a) * (1.0 - sigmoid(a))
    }

    pub fn scaled_translated_sigmoid(a: f64) -> f64 {
        2.0 * sigmoid(a) - 1.0
    }

    pub fn scaled_translated_sigmoid_derivative(a: f64) -> f64 {
        2.0 * sigmoid_derivative(a)
    }
}

pub struct Network {
    hidden_layer_1: FullyConnectedLayer<768, 64>,
    hidden_layer_2: FullyConnectedLayer<64, 16>,
    output_layer: FullyConnectedLayer<16, 1>,
}

impl Network {
    pub fn new_random(rng: &mut dyn RngCore) -> Self {
        Self {
            hidden_layer_1: FullyConnectedLayer::random(rng, ActivationFunction::CLIPPED_RELU),
            hidden_layer_2: FullyConnectedLayer::random(rng, ActivationFunction::CLIPPED_RELU),
            output_layer: FullyConnectedLayer::random(
                rng,
                ActivationFunction::SCALED_TRANSLATED_SIGMOID,
            ),
        }
    }

    pub fn apply(&self, input: &Input) -> f64 {
        let output = self.hidden_layer_1.apply(&input.inner);
        let output = self.hidden_layer_2.apply(&output);
        let output = self.output_layer.apply(&output);
        output[0]
    }

    pub fn to_trainable_network(self) -> TrainableNetwork {
        TrainableNetwork {
            hidden_layer_1: TrainableFullyConnectedLayer::new(self.hidden_layer_1),
            hidden_layer_2: TrainableFullyConnectedLayer::new(self.hidden_layer_2),
            output_layer: TrainableFullyConnectedLayer::new(self.output_layer),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainableNetwork {
    hidden_layer_1: TrainableFullyConnectedLayer<768, 64>,
    hidden_layer_2: TrainableFullyConnectedLayer<64, 16>,
    output_layer: TrainableFullyConnectedLayer<16, 1>,
}

impl TrainableNetwork {
    pub fn to_network(self) -> Network {
        Network {
            hidden_layer_1: self.hidden_layer_1.fcl,
            hidden_layer_2: self.hidden_layer_2.fcl,
            output_layer: self.output_layer.fcl,
        }
    }

    fn apply(&mut self, inputs: &Input) -> HeapArray<f64, 1> {
        let output = self.hidden_layer_1.apply(&inputs.inner);
        let output = self.hidden_layer_2.apply(&output);
        self.output_layer.apply(&output)
    }

    // TODO batches
    // TODO handle biases
    pub fn train<'a>(
        &mut self,
        learning_rate: f64,
        examples: impl Iterator<Item = &'a (Input, f64)>,
    ) -> f64 {
        // TODO Rayon?
        self.hidden_layer_1.clear();
        self.hidden_layer_2.clear();
        self.output_layer.clear();
        let mut count: usize = 0;
        let mut sum = 0.0;
        let mut average_input = HeapArray::zeroed();
        let mut average_error = 0.0;
        for (input, expected) in examples {
            count += 1;
            array_add_assign(&mut average_input, &input.inner);
            // TODO this doesn't handle batches yet
            let result = self.apply(input);
            let expected = HeapArray::new(vec![*expected]);
            average_error += error_function(&result, &expected);
            let dc_da_1 = error_derivative(&result, &expected);
            sum += dc_da_1
        }
        let count_f64 = count as f64;
        average_error /= count_f64;
        self.output_layer.average_from_count(count);
        self.hidden_layer_2.average_from_count(count);
        self.hidden_layer_1.average_from_count(count);
        average_input /= count_f64;
        let dc_da = HeapArray::new(vec![sum / (count as f64)]);

        let dw_output = gradw_c(&dc_da, &self.hidden_layer_2.activations);

        let (delta, dw_hidden_2) = layer(
            &dc_da,
            &self.output_layer.fcl.input_weights,
            &self.hidden_layer_2.derivatives,
            &self.hidden_layer_1.activations,
        );
        let (_delta, dw_hidden_1) = layer(
            &delta,
            &self.hidden_layer_2.fcl.input_weights,
            &self.hidden_layer_1.derivatives,
            &average_input,
        );

        matrix_minus_assign(
            &mut self.output_layer.fcl.input_weights,
            &(dw_output * learning_rate),
        );
        matrix_minus_assign(
            &mut self.hidden_layer_2.fcl.input_weights,
            &(dw_hidden_2 * learning_rate),
        );
        matrix_minus_assign(
            &mut self.hidden_layer_1.fcl.input_weights,
            &(dw_hidden_1 * learning_rate),
        );

        average_error
    }
}

fn layer<const NEXT_NEURONS: usize, const NEURONS: usize, const INPUTS: usize>(
    next_delta: &HeapArray<f64, NEXT_NEURONS>,
    next_weights: &HeapMatrix<f64, NEXT_NEURONS, NEURONS>,
    derivatives: &HeapArray<f64, NEURONS>,
    activations: &HeapArray<f64, INPUTS>,
) -> (HeapArray<f64, NEURONS>, HeapMatrix<f64, NEURONS, INPUTS>) {
    let delta = derivatives.hadamard(&weights_transpose_times_delta(next_weights, next_delta));
    let dw = gradw_c(&delta, activations);
    (delta, dw)
}

fn gradw_c<const INPUTS: usize, const NEURONS: usize>(
    delta: &HeapArray<f64, NEURONS>,
    activations: &HeapArray<f64, INPUTS>,
) -> HeapMatrix<f64, NEURONS, INPUTS> {
    let mut out = HeapMatrix::zeroed();
    for i in 0..NEURONS {
        for j in 0..INPUTS {
            out[i][j] = delta[i] * activations[j]
        }
    }
    out
}

fn weights_transpose_times_delta<const INPUTS: usize, const NEURONS: usize>(
    weights: &HeapMatrix<f64, NEURONS, INPUTS>,
    delta: &HeapArray<f64, NEURONS>,
) -> HeapArray<f64, INPUTS> {
    let mut out = HeapArray::zeroed();
    for j in 0..INPUTS {
        let mut sum = 0.0;
        for k in 0..NEURONS {
            sum += weights[k][j] * delta[k]
        }
        out[j] = sum
    }
    out
}

fn matrix_minus_assign<const M: usize, const N: usize>(
    a: &mut HeapMatrix<f64, M, N>,
    b: &HeapMatrix<f64, M, N>,
) {
    for i in 0..M {
        for j in 0..N {
            a[i][j] -= b[i][j]
        }
    }
}

fn array_add_assign<const N: usize>(a: &mut [f64; N], b: &[f64; N]) {
    for i in 0..N {
        a[i] += b[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn input_indices_are_correct() {
        let mut a = [0; 768];
        for p in Piece::ALL {
            for c in Color::ALL {
                for s in Square::ALL {
                    a[Input::index_for(s, p, c)] += 1
                }
            }
        }
        assert!(a.into_iter().all(|i| i == 1))
    }

    #[test]
    fn apply_network() {
        let mut rand = thread_rng();
        let network = Box::new(Network {
            hidden_layer_1: FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::CLIPPED_RELU,
            ),
            hidden_layer_2: FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::CLIPPED_RELU,
            ),
            output_layer: FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::SCALED_TRANSLATED_SIGMOID,
            ),
        });
        let input = {
            let mut inner = HeapArray::zeroed();
            rand.fill(&mut inner);
            Input { inner }
        };
        let res = network.apply(&input);
        assert_ne!(res, 0.0)
    }

    #[test]
    fn train_network() {
        let mut rand = thread_rng();
        let mut network = Box::new(TrainableNetwork {
            hidden_layer_1: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::RELU,
            )),
            hidden_layer_2: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::RELU,
            )),
            output_layer: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::SCALED_TRANSLATED_SIGMOID,
            )),
        });
        let original = network.clone();
        let input = {
            let mut inner = HeapArray::zeroed();
            rand.fill(&mut inner);
            Input { inner }
        };
        network.train(0.0, vec![(input, -100.0)].iter());
        assert_ne!(
            network.output_layer.fcl.input_weights, original.output_layer.fcl.input_weights,
            "output"
        );
        assert_ne!(
            network.hidden_layer_2.fcl.input_weights, original.hidden_layer_2.fcl.input_weights,
            "hl2"
        );
        assert_ne!(
            network.hidden_layer_1.fcl.input_weights, original.hidden_layer_1.fcl.input_weights,
            "hl1"
        );
    }
}
