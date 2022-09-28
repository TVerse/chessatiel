pub mod data_structures;

use crate::neural_networks::data_structures::{HeapMatrix, HeapVector};
use guts::{Color, Piece, Position, Square};
use num_traits::Inv;
use rand::{Rng, RngCore};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Input<const IN: usize> {
    inner: HeapVector<f64, IN>,
}

impl<const IN: usize> Input<IN> {
    pub fn new(inner: Vec<f64>) -> Self {
        let inner = HeapVector::new(inner);
        Self { inner }
    }
}

impl Input<768> {
    pub fn from_position(pos: &Position) -> Self {
        let mut inner = HeapVector::new(vec![0.0; 768]);

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

    pub fn from_array(inner: HeapVector<f64, 768>) -> Self {
        Self { inner }
    }

    fn index_for(s: Square, p: Piece, c: Color) -> usize {
        s.bitboard_index() + 64 * p.index() + 64 * 6 * c.index()
    }
}

pub trait Layer<T, const INPUTS: usize, const NEURONS: usize> {
    fn apply(&self, input: &HeapVector<T, INPUTS>) -> HeapVector<T, NEURONS>;
}

pub trait TrainableLayer<T, const INPUTS: usize, const NEURONS: usize> {
    fn apply(&mut self, input: &HeapVector<T, INPUTS>) -> HeapVector<T, NEURONS>;

    fn average_from_count(&mut self, count: usize);

    fn activations(&self) -> &HeapVector<T, NEURONS>;

    fn derivatives(&self) -> &HeapVector<T, NEURONS>;

    fn clear(&mut self);
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FullyConnectedLayer<const INPUTS: usize, const NEURONS: usize> {
    input_weights: HeapMatrix<f64, NEURONS, INPUTS>,
    bias_weights: HeapVector<f64, NEURONS>,
    activation_function: ActivationFunction,
}

impl<const INPUTS: usize, const NEURONS: usize> FullyConnectedLayer<INPUTS, NEURONS> {
    pub fn zeroed(activation_function: ActivationFunction) -> Self {
        Self {
            input_weights: HeapMatrix::zeroed(),
            bias_weights: HeapVector::zeroed(),
            activation_function,
        }
    }

    pub fn random(rng: &mut dyn RngCore, activation_function: ActivationFunction) -> Self {
        let mut input_weights = HeapMatrix::zeroed();
        let mut bias_weights = HeapVector::zeroed();
        rng.fill(&mut input_weights);
        rng.fill(&mut bias_weights);
        let input_weights = input_weights * 0.1 - 0.05;
        let bias_weights = bias_weights * 0.1 - 0.05;
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
    fn apply(&self, input: &HeapVector<f64, INPUTS>) -> HeapVector<f64, NEURONS> {
        (&self.input_weights * input + &self.bias_weights)
            .apply(self.activation_function.activation_fn())
    }
}

#[derive(Debug, Clone)]
pub struct TrainableFullyConnectedLayer<const INPUTS: usize, const NEURONS: usize> {
    fcl: FullyConnectedLayer<INPUTS, NEURONS>,
    activations: HeapVector<f64, NEURONS>,
    derivatives: HeapVector<f64, NEURONS>,
}

impl<const INPUTS: usize, const NEURONS: usize> TrainableFullyConnectedLayer<INPUTS, NEURONS> {
    pub fn new(fcl: FullyConnectedLayer<INPUTS, NEURONS>) -> Self {
        Self {
            fcl,
            activations: HeapVector::zeroed(),
            derivatives: HeapVector::zeroed(),
        }
    }
}

impl<const INPUTS: usize, const NEURONS: usize> TrainableLayer<f64, INPUTS, NEURONS>
    for TrainableFullyConnectedLayer<INPUTS, NEURONS>
{
    fn apply(&mut self, input: &HeapVector<f64, INPUTS>) -> HeapVector<f64, NEURONS> {
        let z = &self.fcl.input_weights * input + &self.fcl.bias_weights;
        let out = z
            .clone()
            .apply(self.fcl.activation_function.activation_fn());
        let derivatives = z.apply(self.fcl.activation_function.derivative());

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

    fn activations(&self) -> &HeapVector<f64, NEURONS> {
        &self.activations
    }

    fn derivatives(&self) -> &HeapVector<f64, NEURONS> {
        &self.derivatives
    }

    fn clear(&mut self) {
        self.activations = HeapVector::zeroed();
        self.derivatives = HeapVector::zeroed();
    }
}

pub fn mean_squared_error<const N: usize>(
    output: &HeapVector<f64, N>,
    expected: &HeapVector<f64, N>,
) -> f64 {
    (output.clone() - expected).squared_size() / (2.0 * (N as f64))
}

pub fn mean_squared_error_derivative<const N: usize>(
    output: &HeapVector<f64, N>,
    expected: &HeapVector<f64, N>,
) -> HeapVector<f64, N> {
    (output.clone() - expected) / (N as f64)
}

pub fn cross_entropy<const N: usize>(
    output: &HeapVector<f64, N>,
    expected: &HeapVector<f64, N>,
) -> f64 {
    -output.clone().apply(f64::ln).dot(expected)
}

pub fn cross_entropy_derivative<const N: usize>(
    output: &HeapVector<f64, N>,
    expected: &HeapVector<f64, N>,
) -> HeapVector<f64, N> {
    output
        .clone()
        .apply(f64::inv)
        .hadamard(expected)
        .apply(|f| -f)
}

pub fn binary_cross_entropy(output: &HeapVector<f64, 1>, expected: &HeapVector<f64, 1>) -> f64 {
    let output = output.to_vec()[0];
    let expected = expected.to_vec()[0];
    -(expected * f64::ln(output) + (1.0 - expected) * f64::ln(1.0 - output))
}

pub fn binary_cross_entropy_derivative(
    output: &HeapVector<f64, 1>,
    expected: &HeapVector<f64, 1>,
) -> HeapVector<f64, 1> {
    let output = output.to_vec()[0];
    let expected = expected.to_vec()[0];
    -HeapVector::one(expected / output + (1.0 - expected) / (1.0 - output))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActivationFunction {
    ClippedRelu,
    Sigmoid,
    ScaledTranslatedSigmoid,
    Relu,
    Tanh,
    Linear,
}

impl ActivationFunction {
    pub fn activation_fn(&self) -> fn(f64) -> f64 {
        match self {
            ActivationFunction::ClippedRelu => activation_functions::clipped_relu,
            ActivationFunction::Sigmoid => activation_functions::sigmoid,
            ActivationFunction::ScaledTranslatedSigmoid => {
                activation_functions::scaled_translated_sigmoid
            }
            ActivationFunction::Relu => activation_functions::relu,
            ActivationFunction::Tanh => activation_functions::tanh,
            ActivationFunction::Linear => activation_functions::linear,
        }
    }

    pub fn derivative(&self) -> fn(f64) -> f64 {
        match self {
            ActivationFunction::ClippedRelu => activation_functions::clipped_relu_derivative,
            ActivationFunction::Sigmoid => activation_functions::sigmoid_derivative,
            ActivationFunction::ScaledTranslatedSigmoid => {
                activation_functions::scaled_translated_sigmoid_derivative
            }
            ActivationFunction::Relu => activation_functions::relu_derivative,
            ActivationFunction::Tanh => activation_functions::tanh_derivative,
            ActivationFunction::Linear => activation_functions::linear_derivative,
        }
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

    pub fn tanh(a: f64) -> f64 {
        a.tanh()
    }

    pub fn tanh_derivative(a: f64) -> f64 {
        1.0 / (a.cosh().powi(2))
    }

    pub fn linear(a: f64) -> f64 {
        a
    }

    pub fn linear_derivative(_: f64) -> f64 {
        1.0
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TwoHiddenLayerNetwork<
    const IN: usize,
    const HL1: usize,
    const HL2: usize,
    const OUT: usize,
> {
    hidden_layer_1: FullyConnectedLayer<IN, HL1>,
    hidden_layer_2: FullyConnectedLayer<HL1, HL2>,
    output_layer: FullyConnectedLayer<HL2, OUT>,
}

impl<const IN: usize, const HL1: usize, const HL2: usize, const OUT: usize>
    TwoHiddenLayerNetwork<IN, HL1, HL2, OUT>
{
    pub fn new_random(rng: &mut dyn RngCore) -> Self {
        Self {
            hidden_layer_1: FullyConnectedLayer::random(rng, ActivationFunction::Sigmoid),
            hidden_layer_2: FullyConnectedLayer::random(rng, ActivationFunction::Sigmoid),
            output_layer: FullyConnectedLayer::random(rng, ActivationFunction::Sigmoid),
        }
    }

    pub fn from_bincode(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("Bincode for NN was invalid")
    }

    pub fn apply(&self, input: &Input<IN>) -> HeapVector<f64, OUT> {
        let output = self.hidden_layer_1.apply(&input.inner);
        let output = self.hidden_layer_2.apply(&output);
        self.output_layer.apply(&output)
    }

    pub fn to_trainable_network(
        self,
        cost_function: fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> f64,
        cost_function_gradient: fn(
            &HeapVector<f64, OUT>,
            &HeapVector<f64, OUT>,
        ) -> HeapVector<f64, OUT>,
    ) -> TrainableTwoHiddenLayerNetwork<IN, HL1, HL2, OUT> {
        TrainableTwoHiddenLayerNetwork {
            hidden_layer_1: TrainableFullyConnectedLayer::new(self.hidden_layer_1),
            hidden_layer_2: TrainableFullyConnectedLayer::new(self.hidden_layer_2),
            output_layer: TrainableFullyConnectedLayer::new(self.output_layer),
            cost_function,
            cost_function_gradient,
        }
    }
}

#[derive(Clone)]
pub struct TrainableTwoHiddenLayerNetwork<
    const IN: usize,
    const HL1: usize,
    const HL2: usize,
    const OUT: usize,
> {
    hidden_layer_1: TrainableFullyConnectedLayer<IN, HL1>,
    hidden_layer_2: TrainableFullyConnectedLayer<HL1, HL2>,
    output_layer: TrainableFullyConnectedLayer<HL2, OUT>,
    cost_function: fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> f64,
    cost_function_gradient:
        fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> HeapVector<f64, OUT>,
}

impl<const IN: usize, const HL1: usize, const HL2: usize, const OUT: usize> Debug
    for TrainableTwoHiddenLayerNetwork<IN, HL1, HL2, OUT>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrainableTwoHiddenLayerNetwork")
            .field("hidden_layer_1", &self.hidden_layer_1)
            .field("hidden_layer_2", &self.hidden_layer_2)
            .field("output_layer", &self.output_layer)
            .finish()
    }
}

impl<const IN: usize, const HL1: usize, const HL2: usize, const OUT: usize>
    TrainableTwoHiddenLayerNetwork<IN, HL1, HL2, OUT>
{
    pub fn to_network(self) -> TwoHiddenLayerNetwork<IN, HL1, HL2, OUT> {
        TwoHiddenLayerNetwork {
            hidden_layer_1: self.hidden_layer_1.fcl,
            hidden_layer_2: self.hidden_layer_2.fcl,
            output_layer: self.output_layer.fcl,
        }
    }

    fn apply(&mut self, inputs: &Input<IN>) -> HeapVector<f64, OUT> {
        let output = self.hidden_layer_1.apply(&inputs.inner);
        let output = self.hidden_layer_2.apply(&output);
        self.output_layer.apply(&output)
    }

    // TODO handle biases
    pub fn train<'a>(
        &mut self,
        learning_rate: f64,
        examples: impl Iterator<Item = &'a (Input<IN>, HeapVector<f64, OUT>)>,
    ) -> f64 {
        // TODO Rayon?
        self.hidden_layer_1.clear();
        self.hidden_layer_2.clear();
        self.output_layer.clear();
        let mut count: usize = 0;
        let mut gradient_sum: HeapVector<f64, OUT> = HeapVector::zeroed();
        let mut average_input = HeapVector::zeroed();
        let mut average_error = 0.0;
        for (input, expected) in examples {
            count += 1;
            average_input += &input.inner;
            let result = self.apply(input);
            average_error += (self.cost_function)(&result, expected);
            let dc_da_1 = (self.cost_function_gradient)(&result, expected);
            gradient_sum += &dc_da_1
        }
        let count_f64 = count as f64;
        average_error /= count_f64;
        self.output_layer.average_from_count(count);
        self.hidden_layer_2.average_from_count(count);
        self.hidden_layer_1.average_from_count(count);
        average_input /= count_f64;
        let delta = (gradient_sum / count_f64).hadamard(&self.output_layer.derivatives);

        let activations = &self.hidden_layer_2.activations;
        let dw_output = dbg!(delta.product_to_matrix(activations));

        let (delta, dw_hidden_2) = dbg!(layer(
            &delta,
            &self.output_layer.fcl.input_weights,
            &self.hidden_layer_2.derivatives,
            &self.hidden_layer_1.activations,
        ));
        let (_delta, dw_hidden_1) = dbg!(layer(
            &delta,
            &self.hidden_layer_2.fcl.input_weights,
            &self.hidden_layer_1.derivatives,
            &average_input,
        ));

        self.output_layer.fcl.input_weights -= &(dw_output * learning_rate);
        self.hidden_layer_2.fcl.input_weights -= &(dw_hidden_2 * learning_rate);
        self.hidden_layer_1.fcl.input_weights -= &(dw_hidden_1 * learning_rate);

        average_error
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoHiddenLayerNetwork<const IN: usize, const OUT: usize> {
    output_layer: FullyConnectedLayer<IN, OUT>,
}

impl<const IN: usize, const OUT: usize> NoHiddenLayerNetwork<IN, OUT> {
    pub fn new_random(rng: &mut dyn RngCore, activation_function: ActivationFunction) -> Self {
        Self {
            output_layer: FullyConnectedLayer::random(rng, activation_function),
        }
    }

    pub fn apply(&self, input: &Input<IN>) -> HeapVector<f64, OUT> {
        self.output_layer.apply(&input.inner)
    }

    pub fn to_trainable_network(
        self,
        cost_function: fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> f64,
        cost_function_gradient: fn(
            &HeapVector<f64, OUT>,
            &HeapVector<f64, OUT>,
        ) -> HeapVector<f64, OUT>,
    ) -> TrainableNoHiddenLayerNetwork<IN, OUT> {
        TrainableNoHiddenLayerNetwork {
            output_layer: TrainableFullyConnectedLayer::new(self.output_layer),
            cost_function,
            cost_function_gradient,
        }
    }
}

pub struct TrainableNoHiddenLayerNetwork<const IN: usize, const OUT: usize> {
    output_layer: TrainableFullyConnectedLayer<IN, OUT>,
    cost_function: fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> f64,
    cost_function_gradient:
        fn(&HeapVector<f64, OUT>, &HeapVector<f64, OUT>) -> HeapVector<f64, OUT>,
}

impl<const IN: usize, const OUT: usize> Debug for TrainableNoHiddenLayerNetwork<IN, OUT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrainableNoHiddenLayerNetwork")
            .field("output_layer", &self.output_layer)
            .finish()
    }
}

impl<const IN: usize, const OUT: usize> TrainableNoHiddenLayerNetwork<IN, OUT> {
    pub fn to_network(self) -> NoHiddenLayerNetwork<IN, OUT> {
        NoHiddenLayerNetwork {
            output_layer: self.output_layer.fcl,
        }
    }

    fn apply(&mut self, inputs: &Input<IN>) -> HeapVector<f64, OUT> {
        self.output_layer.apply(&inputs.inner)
    }

    pub fn train<'a>(
        &mut self,
        learning_rate: f64,
        examples: impl Iterator<Item = &'a (Input<IN>, HeapVector<f64, OUT>)>,
    ) -> f64 {
        // TODO Rayon?
        self.output_layer.clear();
        let mut count: usize = 0;
        let mut cost_function_gradient_total: HeapVector<f64, OUT> = HeapVector::zeroed();
        let mut average_input = HeapVector::zeroed();
        let mut average_error = 0.0;
        for (input, expected) in examples {
            count += 1;
            average_input += &input.inner;
            let result = dbg!(self.apply(dbg!(input)));
            average_error += (self.cost_function)(&result, expected);
            let dc_da_1 = dbg!((self.cost_function_gradient)(&result, expected));
            cost_function_gradient_total += &dc_da_1
        }
        let count_f64 = count as f64;
        average_error /= count_f64;
        self.output_layer.average_from_count(count);
        average_input /= count_f64;
        dbg!(&average_input);
        let cost_function_gradient = dbg!(cost_function_gradient_total / count_f64);
        let delta = dbg!((cost_function_gradient).hadamard(dbg!(&self.output_layer.derivatives)));

        let dw_output = dbg!(delta.product_to_matrix(&average_input));

        self.output_layer.fcl.input_weights -= dbg!(&(dw_output * learning_rate));
        dbg!(&self.output_layer.fcl.bias_weights);
        self.output_layer.fcl.bias_weights -= dbg!(&(delta * learning_rate));

        average_error
    }
}
/*
in:
next_delta: delta l+1
next_weights: W l+1
derivatives: f' l
activations: a l-1

out:
delta l
grad_c_w l
 */
fn layer<const NEXT_NEURONS: usize, const NEURONS: usize, const INPUTS: usize>(
    next_delta: &HeapVector<f64, NEXT_NEURONS>,
    next_weights: &HeapMatrix<f64, NEXT_NEURONS, NEURONS>,
    derivatives: &HeapVector<f64, NEURONS>,
    activations: &HeapVector<f64, INPUTS>,
) -> (HeapVector<f64, NEURONS>, HeapMatrix<f64, NEURONS, INPUTS>) {
    let delta = derivatives
        .clone()
        .hadamard(&next_weights.mul_transposed(next_delta));
    let dw = delta.product_to_matrix(activations);
    (delta, dw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::prelude::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

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
        let mut rand = rand_chacha::ChaCha12Rng::seed_from_u64(std::f64::consts::PI.to_bits());
        let network = Box::new(TwoHiddenLayerNetwork::<16, 4, 4, 1> {
            hidden_layer_1: FullyConnectedLayer::random(&mut rand, ActivationFunction::Relu),
            hidden_layer_2: FullyConnectedLayer::random(&mut rand, ActivationFunction::Relu),
            output_layer: FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::ScaledTranslatedSigmoid,
            ),
        });
        let input = {
            let mut inner = HeapVector::zeroed();
            rand.fill(&mut inner);
            Input { inner }
        };
        let res = network.apply(&input)[0];
        assert_ne!(res, 0.0);
        let second_input = {
            let mut inner = HeapVector::zeroed();
            rand.fill(&mut inner);
            Input { inner }
        };
        assert_ne!(input, second_input);
        let second_res = network.apply(&second_input)[0];
        assert_ne!(res, second_res)
    }

    #[test]
    fn train_network() {
        let mut rand = ChaCha20Rng::seed_from_u64(std::f64::consts::LN_2.to_bits());
        let mut network = TrainableTwoHiddenLayerNetwork::<16, 4, 4, 1> {
            hidden_layer_1: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::Relu,
            )),
            hidden_layer_2: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::Relu,
            )),
            output_layer: TrainableFullyConnectedLayer::new(FullyConnectedLayer::random(
                &mut rand,
                ActivationFunction::ScaledTranslatedSigmoid,
            )),
            cost_function: mean_squared_error,
            cost_function_gradient: mean_squared_error_derivative,
        };
        let original = network.clone();
        let input = {
            let mut inner = HeapVector::zeroed();
            rand.fill(&mut inner);
            Input { inner }
        };
        network.train(100.0, vec![(input, HeapVector::new(vec![-100.0]))].iter());
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

    #[test]
    fn sigmoid() {
        let sigmoid = activation_functions::sigmoid;
        assert_eq!(sigmoid(0.0), 0.5);
        assert!(sigmoid(10.0) > 0.99);
        assert!(sigmoid(-10.0) < 0.01);
    }

    #[test]
    fn train_is_pos_function() {
        let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(1234);
        let inputs = vec![
            vec![-9.0],
            vec![-8.0],
            vec![-7.0],
            vec![-6.0],
            vec![-5.0],
            vec![-4.0],
            vec![-3.0],
            vec![-2.0],
            vec![-1.0],
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec![4.0],
            vec![5.0],
            vec![6.0],
            vec![7.0],
            vec![8.0],
            vec![9.0],
            vec![10.0],
        ];
        let outputs = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0,
        ];
        assert_eq!(inputs.len(), outputs.len());
        let training_set = inputs
            .clone()
            .into_iter()
            .map(Input::<1>::new)
            .zip(
                outputs
                    .into_iter()
                    .map(|t| HeapVector::<f64, 1>::new(vec![t])),
            )
            .collect_vec();
        let network =
            NoHiddenLayerNetwork::<1, 1>::new_random(&mut rng, ActivationFunction::Sigmoid);
        let mut trainable_network =
            network.to_trainable_network(binary_cross_entropy, binary_cross_entropy_derivative);
        for i in 0..10000 {
            dbg!(i);
            dbg!(&trainable_network);
            let random_set = training_set.choose_multiple(&mut rng, 2);
            let error = trainable_network.train(0.003, random_set);
            dbg!(error);
        }
        let network = trainable_network.to_network();
        for i in inputs {
            dbg!((&i, network.apply(&Input::new(i.clone()))));
        }
        dbg!(&network);
        for (i, o) in training_set.iter() {
            let output = network.apply(i);
            let output = output[0];
            let rounded_output = if output >= 0.5 { 1.0 } else { 0.0 };
            assert_eq!(
                rounded_output, o[0],
                "in: {i:?}, expected: {o:?}, got: {output}"
            );
        }
    }

    #[test]
    fn train_or_function() {
        let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(1234);
        let inputs = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
        ];
        let outputs = vec![0.0, 1.0, 1.0, 1.0];
        assert_eq!(inputs.len(), outputs.len());
        let training_set = inputs
            .clone()
            .into_iter()
            .map(Input::<2>::new)
            .zip(
                outputs
                    .into_iter()
                    .map(|t| HeapVector::<f64, 1>::new(vec![t])),
            )
            .collect_vec();
        let network =
            NoHiddenLayerNetwork::<2, 1>::new_random(&mut rng, ActivationFunction::Sigmoid);
        let mut trainable_network =
            network.to_trainable_network(binary_cross_entropy, binary_cross_entropy_derivative);
        for _ in 0..10000 {
            dbg!(&trainable_network);
            let error = trainable_network.train(1.0, training_set.iter());
            dbg!(error);
        }
        let network = trainable_network.to_network();
        for i in inputs {
            dbg!((&i, network.apply(&Input::new(i.clone()))));
        }
        dbg!(&network);
        for (i, o) in training_set.iter() {
            let output = network.apply(i);
            let output = output[0];
            let rounded_output = if output >= 0.5 { 1.0 } else { 0.0 };
            assert_eq!(
                rounded_output, o[0],
                "in: {i:?}, expected: {o:?}, got: {output}"
            );
        }
    }

    fn test_derivative(desc: &str, f: impl Fn(f64) -> f64, df: impl Fn(f64) -> f64) {
        let test_points = (1..100)
            .chain((110..=1000).step_by(10))
            .map(|i| i as f64 * 0.01);
        let numeric_derivative_at = |a: f64| -> f64 {
            let neg = f(a - 0.0001);
            let plu = f(a + 0.0001);
            (plu - neg) / 0.0002
        };
        for t in test_points {
            let exact_derivative = df(t);
            let numeric_derivative = numeric_derivative_at(t);
            assert!(
                (exact_derivative - numeric_derivative).abs() < 0.00001,
                "Failed on {desc} at {t}. Exact: {exact_derivative}, numeric: {numeric_derivative}"
            );
            let exact_derivative = df(-t);
            let numeric_derivative = numeric_derivative_at(-t);
            assert!(
                (exact_derivative - numeric_derivative).abs() < 0.00001,
                "Failed on {desc} at {t}. Exact: {exact_derivative}, numeric: {numeric_derivative}"
            );
        }
    }

    fn test_activation_fn_derivative(activation_function: ActivationFunction) {
        test_derivative(
            &format!("{:?}", activation_function),
            activation_function.activation_fn(),
            activation_function.derivative(),
        )
    }

    #[test]
    fn test_activation_fn_derivatives() {
        test_activation_fn_derivative(ActivationFunction::ClippedRelu);
        test_activation_fn_derivative(ActivationFunction::Sigmoid);
        test_activation_fn_derivative(ActivationFunction::ScaledTranslatedSigmoid);
        test_activation_fn_derivative(ActivationFunction::Relu);
        test_activation_fn_derivative(ActivationFunction::Tanh);
        test_activation_fn_derivative(ActivationFunction::Linear);
    }

    fn cost_gradient<const N: usize>(
        desc: &str,
        rng: &mut dyn RngCore,
        f: impl Fn(&HeapVector<f64, N>, &HeapVector<f64, N>) -> f64,
        df: impl Fn(&HeapVector<f64, N>, &HeapVector<f64, N>) -> HeapVector<f64, N>,
    ) {
        let numeric_gradient_at =
            |o: &HeapVector<f64, N>, e: &HeapVector<f64, N>| -> HeapVector<f64, N> {
                let mut sum = vec![0.0; N];
                for i in 0..N {
                    let h = 0.00001;
                    let mut perturbed_pos = o.to_vec().clone();
                    perturbed_pos[i] += h;
                    let mut perturbed_neg = o.to_vec().clone();
                    perturbed_neg[i] -= h;
                    let pos = f(&HeapVector::new(perturbed_pos), e);
                    let neg = f(&HeapVector::new(perturbed_neg), e);
                    sum[i] = (pos - neg) / (2.0 * h);
                }
                HeapVector::new(sum)
            };
        let expected = {
            let mut expected = Vec::with_capacity(N);
            for i in 0..N {
                let mut e = vec![0.0; N];
                e[i] = 1.0;
                expected.push(HeapVector::new(e))
            }
            expected
        };
        let output = {
            let mut output = Vec::with_capacity(20);
            for _i in 0..20 {
                let mut o = vec![0.0; N];
                rng.fill(&mut o[..]);
                output.push(HeapVector::new(o))
            }
            output
        };
        for e in expected.into_iter() {
            for o in output.iter() {
                let exact_gradient = df(o, &e);
                let numeric_gradient = numeric_gradient_at(o, &e);
                let diff = (exact_gradient - &numeric_gradient).apply(f64::abs);
                for i in diff.to_vec() {
                    assert!(
                        *i < 0.00001,
                        "For {desc},got diff {diff:?}. Inputs: o: {o:?}, e: {e:?}"
                    )
                }
            }
        }
    }

    #[test]
    fn cost_mse() {
        const N: usize = 5;
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::SQRT_2.to_bits());

        let f = mean_squared_error::<N>;
        let df = mean_squared_error_derivative::<N>;

        cost_gradient("mean_squared_error", &mut rng, f, df);
    }

    #[test]
    fn gradient_mse_1() {
        const N: usize = 5;

        let f = mean_squared_error::<N>;
        let df = mean_squared_error_derivative::<N>;

        let o = HeapVector::<f64, N>::new(vec![
            0.4293023846652214,
            0.7597686114660552,
            0.30802177624847515,
            0.5723299096412179,
            0.034502539506032326,
        ]);
        let e = HeapVector::<f64, N>::new(vec![1.0, 0.0, 0.0, 0.0, 0.0]);
        let numeric_gradient_at =
            |o: &HeapVector<f64, N>, e: &HeapVector<f64, N>| -> HeapVector<f64, N> {
                let mut sum = vec![0.0; N];
                for i in 0..N {
                    let h = 0.0001;
                    let mut perturbed_pos = o.to_vec().clone();
                    perturbed_pos[i] += h;
                    let mut perturbed_neg = o.to_vec().clone();
                    perturbed_neg[i] -= h;
                    let pos = f(&HeapVector::new(perturbed_pos), e);
                    let neg = f(&HeapVector::new(perturbed_neg), e);
                    sum[i] = (pos - neg) / (2.0 * h);
                }
                HeapVector::new(sum)
            };
        let exact_gradient = df(&o, &e);
        let numeric_gradient = numeric_gradient_at(&o, &e);
        dbg!(&exact_gradient);
        dbg!(&numeric_gradient);
        let diff = (exact_gradient - &numeric_gradient).apply(f64::abs);
        for i in diff.to_vec() {
            assert!(
                *i < 0.00001,
                "Got diff {diff:?}. Inputs: o: {o:?}, e: {e:?}"
            )
        }
    }

    #[test]
    fn cost_cross_entropy() {
        const N: usize = 5;
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::SQRT_2.to_bits());

        let f = cross_entropy::<N>;
        let df = cross_entropy_derivative::<N>;

        cost_gradient("cross_entropy", &mut rng, f, df);
    }

    #[test]
    fn gradient_cross_entropy_1() {
        const N: usize = 5;

        let f = cross_entropy::<N>;
        let df = cross_entropy_derivative::<N>;

        let o = HeapVector::<f64, N>::new(vec![
            0.4293023846652214,
            0.7597686114660552,
            0.30802177624847515,
            0.5723299096412179,
            0.034502539506032326,
        ]);
        let e = HeapVector::<f64, N>::new(vec![1.0, 0.0, 0.0, 0.0, 0.0]);
        let numeric_gradient_at =
            |o: &HeapVector<f64, N>, e: &HeapVector<f64, N>| -> HeapVector<f64, N> {
                let mut sum = vec![0.0; N];
                for i in 0..N {
                    let h = 0.00001;
                    let mut perturbed_pos = o.to_vec().clone();
                    perturbed_pos[i] += h;
                    let mut perturbed_neg = o.to_vec().clone();
                    perturbed_neg[i] -= h;
                    let pos = f(&HeapVector::new(perturbed_pos), e);
                    let neg = f(&HeapVector::new(perturbed_neg), e);
                    sum[i] = (pos - neg) / (2.0 * h);
                }
                HeapVector::new(sum)
            };
        let exact_gradient = df(&o, &e);
        let numeric_gradient = numeric_gradient_at(&o, &e);
        dbg!(&exact_gradient);
        dbg!(&numeric_gradient);
        let diff = (exact_gradient - &numeric_gradient).apply(f64::abs);
        for i in diff.to_vec() {
            assert!(
                *i < 0.00001,
                "Got diff {diff:?}. Inputs: o: {o:?}, e: {e:?}"
            )
        }
    }

    #[test]
    fn cost_binary_cross_entropy() {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::SQRT_2.to_bits());

        let f = binary_cross_entropy;
        let df = binary_cross_entropy_derivative;

        cost_gradient("binary_cross_entropy", &mut rng, f, df)
    }
}
