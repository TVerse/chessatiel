use crate::{AnnotatedPosition, GameResult};
use brain::evaluator::pst_evaluator::pst::{dot, PieceSquareTable};
use itertools::Itertools;
use rayon::prelude::*;

type TrainingPair = (Vec<(usize, f64)>, GameResult);

pub fn train_pst(learning_rate: f64, training_set: Vec<AnnotatedPosition>) -> Vec<f64> {
    let mut pst = PieceSquareTable::zeroes();
    let coefficients = pst.values_mut();
    println!(
        "Loaded {} annotated positions ({} bytes)",
        training_set.len(),
        training_set.len() * std::mem::size_of::<AnnotatedPosition>()
    );
    dbg!(&coefficients);
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
    dbg!(&coefficients);
    println!("Starting second full pass");
    update_coefficients(learning_rate, coefficients, &training_set);
    dbg!(&coefficients);

    Vec::from(coefficients)
}

fn update_coefficients(
    learning_rate: f64,
    coefficients: &mut [f64],
    training_set: &[TrainingPair],
) -> f64 {
    let mut subs = Vec::with_capacity(coefficients.len());

    let div = training_set.len() as f64;

    let res = (0..coefficients.len()).into_par_iter().map(|i| {
        let s = training_set
            .iter()
            .map(|(x, y)| {
                derivative_cost_function_one_input(f64::from(*y), x, coefficients, get_sparse(x, i))
            })
            .sum::<f64>();
        s / div
    });
    subs.par_extend(res);

    let size_grad = subs.iter().map(|s| s * s).sum();

    coefficients
        .iter_mut()
        .zip(subs.iter())
        .for_each(|(w, sub)| *w -= learning_rate * sub);

    size_grad
}

fn get_sparse(sparse: &[(usize, f64)], idx: usize) -> f64 {
    // TODO try sorting indices to make this faster
    sparse
        .iter()
        .find(|(i, _)| *i == idx)
        .map(|(_, x)| x)
        .copied()
        .unwrap_or(0.0)
}

fn derivative_cost_function_one_input(
    y: f64,
    input: &[(usize, f64)],
    coefficients: &[f64],
    xi: f64,
) -> f64 {
    let d = evaluate(input, coefficients);
    let f = sigmoid(d);
    // let dsigmoid = (xi / ((1.0 + d * d).powf(1.5)));
    let t = f64::exp(-d) + 1.0;
    let dsigmoid = xi * f64::exp(-d) / (t * t);
    (f - y) * dsigmoid
}

fn sigmoid(x: f64) -> f64 {
    // TODO check if this has influence on strength
    // x / (1.0 + x * x).sqrt()
    2.0 / (1.0 + f64::exp(-x)) - 1.0
}

fn evaluate(input: &[(usize, f64)], coefficients: &[f64]) -> f64 {
    dot(input, coefficients)
}
