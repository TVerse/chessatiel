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

pub fn train_mnist(learning_rate: f64, training_set: &[String], validate_set: &[String]) -> Net {
    // let validate_diff_cutoff = -0.000000001;
    let training_set = convert(training_set);
    let validate_set = convert(validate_set);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
    let mut network = Net::new_random(&mut rng).to_trainable_network();
    let mut prev_training_error = 1.0;
    let mut prev_validate_error = 1.0;
    let train_examples = training_set.iter().cycle().chunks(10_000);
    let mut i = 0;
    for chunk in &train_examples {
        i += 1;
        println!("Iteration {i}");
        let chunk = chunk.collect_vec();
        let (_, _validate_diff) = iteration(
            learning_rate,
            &chunk,
            &validate_set,
            &mut network,
            &mut prev_training_error,
            &mut prev_validate_error,
        );
        // if validate_diff.abs() <= validate_diff_cutoff {
        //     println!("Cutoff reached");
        //     break;
        // }
        if i == 100 {
            break;
        }
    }

    let network = network.to_network();

    for (pos, l) in validate_set {
        println!("{:?}, {:?}", l, network.apply(&pos))
    }

    network
}

fn iteration(
    learning_rate: f64,
    training_set: &[&TrainingPair],
    validate_set: &[TrainingPair],
    network: &mut TrainableNet,
    prev_training_error: &mut f64,
    prev_validate_error: &mut f64,
) -> (f64, f64) {
    let start = Instant::now();
    let train_error = do_train(learning_rate, network, training_set);
    let train_diff = train_error - *prev_training_error;
    *prev_training_error = train_error;
    println!("Training error: {train_error} (diff: {train_diff})");
    let validate_error = do_validate(&network.clone().to_network(), validate_set);
    let validate_diff = validate_error - *prev_validate_error;
    *prev_validate_error = validate_error;
    println!("Validate error: {validate_error} (diff: {validate_diff})");
    let duration = start.elapsed();
    println!("Took: {} ms", duration.as_millis());
    (train_diff, validate_diff)
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

fn do_validate(network: &Net, validate_set: &[TrainingPair]) -> f64 {
    let mut e = 0.0;
    let mut correct = 0;
    for (i, o) in validate_set {
        let res = network.apply(i);
        let actual = get_category(o);
        let predicted = get_category(&res);
        if actual == predicted {
            correct += 1;
        }
        let error = error_function(&res, o);
        e += error
    }
    println!("Correct: {correct} out of {l}", l = validate_set.len());
    e / (validate_set.len() as f64)
}

fn get_category(out: &HeapVector<f64, 10>) -> u8 {
    out.to_vec()
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap() as u8
}
