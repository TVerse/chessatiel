use crate::{AnnotatedPosition, GameResult};
use brain::evaluator::pst_evaluator::pst::{dot, PieceSquareTable};
use itertools::Itertools;
use rayon::prelude::*;

type TrainingPair = (Vec<(usize, f32)>, GameResult);

pub fn train(learning_rate: f32, training_set: Vec<AnnotatedPosition>) -> Vec<f32> {
    let mut pst = PieceSquareTable::zeroes();
    let coefficients = pst.values_mut();
    println!(
        "Loaded {} annotated positions ({} bytes)",
        training_set.len(),
        training_set.len() * std::mem::size_of::<AnnotatedPosition>()
    );
    println!("Starting converting");
    let training_set = training_set
        .into_iter()
        .map(|ap| (PieceSquareTable::position_as_vec(&ap.pos), ap.result))
        .collect_vec();
    println!(
        "Converted {} annotated positions ({} bytes)",
        training_set.len(),
        training_set.len() * std::mem::size_of::<AnnotatedPosition>()
    );
    // TODO more than 1 or 2 full passes might not make sense here
    // The dataset is self-conflicting in places, it'll never fully converge error-wise
    println!("Starting first full pass");
    update_coefficients(learning_rate, coefficients, &training_set);
    println!("Starting second full pass");
    update_coefficients(learning_rate, coefficients, &training_set);

    Vec::from(coefficients)
}

fn update_coefficients(
    learning_rate: f32,
    coefficients: &mut [f32],
    training_set: &[TrainingPair],
) -> f32 {
    let mut subs = Vec::with_capacity(coefficients.len());

    let res = (0..coefficients.len()).into_par_iter().map(|i| {
        training_set
            .iter()
            .map(|(x, y)| {
                derivative_cost_function_one_input(f32::from(*y), x, coefficients, get_sparse(x, i))
            })
            .sum::<f32>()
    });
    subs.par_extend(res);

    let div = training_set.len() as f32;

    let size_grad = subs.iter().map(|s| (s / div) * (s / div)).sum();

    coefficients
        .iter_mut()
        .zip(subs.iter())
        .for_each(|(w, sub)| *w -= learning_rate * sub / div);

    size_grad
}

fn get_sparse(sparse: &[(usize, f32)], idx: usize) -> f32 {
    // TODO try sorting indices to make this faster
    sparse
        .iter()
        .find(|(i, _)| *i == idx)
        .map(|(_, x)| x)
        .copied()
        .unwrap_or(0.0)
}

fn derivative_cost_function_one_input(
    y: f32,
    input: &[(usize, f32)],
    coefficients: &[f32],
    xi: f32,
) -> f32 {
    let d = evaluate(input, coefficients);
    let f = sigmoid(d);
    // let dsigmoid = (xi / ((1.0 + d * d).powf(1.5)));
    let t = f32::exp(-d) + 1.0;
    let dsigmoid = xi * f32::exp(-d) / (t * t);
    (f - y) * dsigmoid
}

fn sigmoid(x: f32) -> f32 {
    // TODO check if this has influence on strength
    // x / (1.0 + x * x).sqrt()
    2.0 / (1.0 + f32::exp(-x)) - 1.0
}

fn evaluate(input: &[(usize, f32)], coefficients: &[f32]) -> f32 {
    dot(input, coefficients)
}
