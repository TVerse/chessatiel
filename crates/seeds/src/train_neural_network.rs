use crate::AnnotatedPosition;
use brain::neural_networks::heap_arrays::HeapArray;
use brain::neural_networks::{error_function, Input, Network, TrainableNetwork};
use itertools::Itertools;
use rand::SeedableRng;
use std::time::Instant;

type TrainingPair = (Input, f64);

pub fn train_nn(
    learning_rate: f64,
    training_set: &[AnnotatedPosition],
    validation_set: &[AnnotatedPosition],
) -> Network {
    let training_set = convert(training_set);
    let validation_set = convert(validation_set);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
    let mut network = Network::new_random(&mut rng).to_trainable_network();
    let mut prev_training_error = 1.0;
    let mut prev_validation_error = 1.0;
    for i in 0..200 {
        println!("Iteration {i}");
        iteration(
            learning_rate,
            &training_set,
            &validation_set,
            &mut network,
            &mut prev_training_error,
            &mut prev_validation_error,
        );
    }

    network.to_network()
}

fn iteration(
    learning_rate: f64,
    training_set: &[TrainingPair],
    validation_set: &[TrainingPair],
    network: &mut TrainableNetwork,
    prev_training_error: &mut f64,
    prev_validation_error: &mut f64,
) {
    let start = Instant::now();
    let train_error = do_train(learning_rate, network, training_set);
    let train_diff = train_error - *prev_training_error;
    *prev_training_error = train_error;
    println!("Training error: {train_error} (diff: {train_diff})");
    let validation_error = do_validate(&network.clone().to_network(), validation_set);
    let validation_diff = validation_error - *prev_validation_error;
    *prev_validation_error = validation_error;
    println!("Validation error: {validation_error} (diff: {validation_diff})");
    let duration = start.elapsed();
    println!("Took: {} ms", duration.as_millis());
}

fn convert(pos: &[AnnotatedPosition]) -> Vec<TrainingPair> {
    println!(
        "Loaded {} annotated positions ({} bytes)",
        pos.len(),
        pos.len() * std::mem::size_of::<AnnotatedPosition>()
    );
    println!("Starting converting");
    let pos = pos
        .iter()
        .map(|ap| (Input::new(&ap.pos), f64::from(ap.result)))
        .collect_vec();
    println!(
        "Converted {} annotated positions ({} bytes)",
        pos.len(),
        pos.len() * std::mem::size_of::<AnnotatedPosition>()
    );
    pos
}

fn do_train(
    learning_rate: f64,
    network: &mut TrainableNetwork,
    training_set: &[TrainingPair],
) -> f64 {
    network.train(learning_rate, training_set.iter())
}

fn do_validate(network: &Network, validation_set: &[TrainingPair]) -> f64 {
    let mut e = 0.0;
    for (i, o) in validation_set {
        let res = network.apply(i);
        let error = error_function::<1>(&HeapArray::new(vec![res]), &HeapArray::new(vec![*o]));
        e += error
    }
    e / (validation_set.len() as f64)
}
