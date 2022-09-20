use brain::neural_networks::heap_arrays::HeapVector;
use brain::neural_networks::{Input, TwoHiddenLayerNetwork};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng, SeedableRng};

fn train_single(c: &mut Criterion) {
    c.bench_function("train_single", |b| {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
        let input = {
            let mut inner = HeapVector::zeroed();
            rng.fill(&mut inner);
            Input::from_array(inner)
        };
        let inputs = vec![(input, HeapVector::new(vec![1.0]))];
        let mut trainable_network =
            TwoHiddenLayerNetwork::<768, 64, 16, 1>::new_random(&mut rng).to_trainable_network();
        b.iter(|| black_box(&mut trainable_network).train(0.1, black_box(&inputs).iter()))
    });
}

fn train_batch(c: &mut Criterion) {
    c.bench_function("train_batch", |b| {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(std::f64::consts::E.to_bits());
        let mut inputs = Vec::with_capacity(100);
        for i in 0..inputs.capacity() {
            let input = {
                let mut inner = HeapVector::zeroed();
                rng.fill(&mut inner);
                Input::from_array(inner)
            };
            let expected = if i % 3 == 0 {
                -1.0
            } else if i % 3 == 1 {
                0.0
            } else {
                1.0
            };
            inputs.push((input, HeapVector::new(vec![expected])));
        }
        let mut trainable_network =
            TwoHiddenLayerNetwork::<768, 64, 16, 1>::new_random(&mut rng).to_trainable_network();
        b.iter(|| black_box(&mut trainable_network).train(0.1, black_box(&inputs).iter()))
    });
}

fn apply(c: &mut Criterion) {
    c.bench_function("apply", |b| {
        let mut rng = thread_rng();
        let input = {
            let mut inner = HeapVector::zeroed();
            rng.fill(&mut inner);
            Input::from_array(inner)
        };
        let network = TwoHiddenLayerNetwork::<768, 64, 16, 1>::new_random(&mut rng);
        b.iter(|| black_box(&network).apply(black_box(&input)))
    });
}

criterion_group! {
    name = neural_networks;
    config = Criterion::default();
    targets = train_single, train_batch, apply
}

criterion_main!(neural_networks);
