use crate::AnnotatedPosition;
use brain::neural_networks::heap_arrays::HeapVector;
use brain::neural_networks::{
    error_function, Input, TrainableTwoHiddenLayerNetwork, TwoHiddenLayerNetwork,
};
use itertools::Itertools;
use rand::SeedableRng;
use std::time::Instant;

type TrainingPair = (Input<768>, HeapVector<f64, 1>);

pub fn train_nn(
    learning_rate: f64,
    training_set: &[AnnotatedPosition],
    test_set: &[AnnotatedPosition],
) -> TwoHiddenLayerNetwork<768, 64, 16, 1> {
    // let test_diff_cutoff = -0.000000001;
    let training_set = convert(training_set);
    let test_set = convert(test_set);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
    let mut network = TwoHiddenLayerNetwork::new_random(&mut rng).to_trainable_network();
    let mut prev_training_error = 1.0;
    let mut prev_test_error = 1.0;
    let train_examples = training_set.iter().cycle().chunks(10_000);
    let mut i = 0;
    for chunk in &train_examples {
        i += 1;
        println!("Iteration {i}");
        let chunk = chunk.collect_vec();
        let (_, _test_diff) = iteration(
            learning_rate,
            &chunk,
            &test_set,
            &mut network,
            &mut prev_training_error,
            &mut prev_test_error,
        );
        // if test_diff.abs() <= test_diff_cutoff {
        //     println!("Cutoff reached");
        //     break;
        // }
        if i == 1_000 {
            break;
        }
    }

    network.to_network()
}

fn iteration(
    learning_rate: f64,
    training_set: &[&TrainingPair],
    test_set: &[TrainingPair],
    network: &mut TrainableTwoHiddenLayerNetwork<768, 64, 16, 1>,
    prev_training_error: &mut f64,
    prev_test_error: &mut f64,
) -> (f64, f64) {
    let start = Instant::now();
    let train_error = do_train(learning_rate, network, training_set);
    let train_diff = train_error - *prev_training_error;
    *prev_training_error = train_error;
    println!("Training error: {train_error} (diff: {train_diff})");
    let test_error = do_test(&network.clone().to_network(), test_set);
    let test_diff = test_error - *prev_test_error;
    *prev_test_error = test_error;
    println!("Test error: {test_error} (diff: {test_diff})");
    let duration = start.elapsed();
    println!("Took: {} ms", duration.as_millis());
    (train_diff, test_diff)
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
        .map(|ap| {
            (
                Input::from_position(&ap.pos),
                HeapVector::new(vec![f64::from(ap.result)]),
            )
        })
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
    network: &mut TrainableTwoHiddenLayerNetwork<768, 64, 16, 1>,
    training_set: &[&TrainingPair],
) -> f64 {
    network.train(learning_rate, training_set.iter().copied())
}

fn do_test(network: &TwoHiddenLayerNetwork<768, 64, 16, 1>, test_set: &[TrainingPair]) -> f64 {
    let mut e = 0.0;
    for (i, o) in test_set {
        let res = network.apply(i);
        let error = error_function::<1>(&res, o);
        e += error
    }
    e / (test_set.len() as f64)
}
