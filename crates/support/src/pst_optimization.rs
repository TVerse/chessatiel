use crate::{AnnotatedPosition, GameResult};
use brain::evaluator::pst_evaluator::pst::{dot, PieceSquareTable};
use itertools::Itertools;
use rayon::prelude::*;

type TrainingPair = (Vec<(usize, f32)>, GameResult);

pub fn train(learning_rate: f32, training_set: &[AnnotatedPosition]) -> Vec<f32> {
    let mut pst = PieceSquareTable::zeroes();
    let coefficients = pst.values_mut();
    let training_set = training_set
        .iter()
        .map(|ap| (PieceSquareTable::position_as_vec(&ap.pos), ap.result))
        .collect_vec();
    println!("Starting full pass");
    update_coefficients(learning_rate, coefficients, &training_set);
    let chunk_size = 64;
    println!("Full pass done, starting minibatch (batch size {chunk_size})");
    let mut positions_done = 0;
    'outer: loop {
        let it = training_set.chunks(chunk_size);
        let mut chunks_this_cycle = 0;
        for batch in it {
            let size_grad = update_coefficients(learning_rate, coefficients, batch);
            positions_done += batch.len();
            chunks_this_cycle += 1;
            if chunks_this_cycle % 100 == 0 {
                println!("|grad|: {size_grad} after {positions_done} total positions");
            }
            // TODO is stopping based on grad a good idea? Probably not.
            // Does this even terminate for small cutoff?
            if size_grad.abs() < 0.001 && size_grad != 0.0 {
                println!("Gradient small, stopping after {positions_done} positions, {chunks_this_cycle} chunks this cycle (approx {} positions)", chunks_this_cycle * chunk_size);
                break 'outer;
            }
        }
    }

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
