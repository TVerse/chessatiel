use brain::evaluator::PieceValueEvaluator;
use brain::position_hash_history::PositionHashHistory;
use brain::searcher::{SearchConfig, Searcher};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::Position;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;

fn search_startpos(c: &mut Criterion) {
    let mut pos = Position::default();
    let mut history = PositionHashHistory::new(pos.hash());
    c.bench_function("search_startpos", |b| {
        b.iter(|| {
            let (_c_tx, c_rx) = watch::channel(());
            let mut searcher = Searcher::with_evaluator_and_config(
                black_box(&mut history),
                black_box(&mut pos),
                c_rx,
                PieceValueEvaluator::new(),
                SearchConfig { depth: 5 },
            );
            let (tx, _rx) = mpsc::unbounded_channel();
            searcher.search(tx);
        })
    });
}

criterion_group! {
    name = search;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = search_startpos
}
criterion_main!(search);
