use brain::evaluator::PieceCountEvaluator;
use brain::position_history::PositionHistory;
use brain::searcher::{SearchConfig, Searcher};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::Position;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;

fn search_startpos_3(c: &mut Criterion) {
    let pos = Position::default();
    let mut history = PositionHistory::new(pos);
    c.bench_function("search_startpos_3", |b| {
        b.iter(|| {
            let (_c_tx, c_rx) = watch::channel(());
            let mut searcher = Searcher::with_evaluator_and_config(
                black_box(&mut history),
                c_rx,
                PieceCountEvaluator::new(),
                SearchConfig { depth: 3 },
            );
            let (tx, _rx) = mpsc::unbounded_channel();
            searcher.search(tx);
        })
    });
}

criterion_group! {
    name = search;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = search_startpos_3
}
criterion_main!(search);
