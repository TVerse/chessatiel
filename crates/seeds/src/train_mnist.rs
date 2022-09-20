use brain::neural_networks::heap_arrays::HeapVector;
use brain::neural_networks::{
    error_function, Input, TrainableTwoHiddenLayerNetwork, TwoHiddenLayerNetwork,
};
use itertools::Itertools;
use rand::SeedableRng;
use std::time::Instant;

type TrainingPair = (Input<784>, HeapVector<f64, 10>);

pub type Net = TwoHiddenLayerNetwork<784, 16, 8, 10>;
pub type TrainableNet = TrainableTwoHiddenLayerNetwork<784, 16, 8, 10>;

pub fn train_mnist(learning_rate: f64, training_set: &[String], test_set: &[String]) -> Net {
    // let test_diff_cutoff = -0.000000001;
    println!("!!!!");
    dbg!(&training_set.iter().take(20));
    let training_set = convert(training_set);
    println!("!! !!");
    dbg!(&training_set.iter().take(20));
    let test_set = convert(test_set);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
    let mut network = Net::new_random(&mut rng).to_trainable_network();
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
        if i == 100 {
            break;
        }
    }

    let network = network.to_network();

    for (pos, l) in test_set {
        println!("{:?}, {:?}", l, network.apply(&pos))
    }

    network
}

fn iteration(
    learning_rate: f64,
    training_set: &[&TrainingPair],
    test_set: &[TrainingPair],
    network: &mut TrainableNet,
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

fn convert(pos: &[String]) -> Vec<TrainingPair> {
    pos.iter()
        .map(|s| {
            let mut split = s.split(',');
            let label: usize = split.next().unwrap().parse().unwrap();
            let mut expected = HeapVector::<f64, 10>::new(vec![0.0; 10]);
            expected[label] = 1.0;
            let values = split.map(|s| s.parse().unwrap()).collect_vec();
            let values = Input::new(values);
            (values, expected)
        })
        .collect_vec()
}

fn do_train(learning_rate: f64, network: &mut TrainableNet, training_set: &[&TrainingPair]) -> f64 {
    network.train(learning_rate, training_set.iter().copied())
}

fn do_test(network: &Net, test_set: &[TrainingPair]) -> f64 {
    let mut e = 0.0;
    for (i, o) in test_set {
        let res = network.apply(i);
        let error = error_function(&res, o);
        e += error
    }
    e / (test_set.len() as f64)
}
