use brain::evaluator::PieceSquareTableEvaluator;
use brain::position_hash_history::PositionHashHistory;
use brain::searcher::{Searcher, SearcherConfig};
use brain::statistics::StatisticsHolder;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::Position;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;

fn search_startpos(c: &mut Criterion) {
    c.bench_function("search_startpos", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    PieceSquareTableEvaluator::new(),
                    SearcherConfig { depth: Some(6) },
                    Arc::new(StatisticsHolder::new()),
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

criterion_group! {
    name = search;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = search_startpos
}
criterion_main!(search);
