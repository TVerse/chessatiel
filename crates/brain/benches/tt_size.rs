use brain::evaluator::MainEvaluator;
use brain::position_hash_history::PositionHashHistory;
use brain::searcher::{Searcher, SearcherConfig};
use brain::statistics::StatisticsHolder;
use brain::transposition_table::TranspositionTable;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::Position;
use tokio::sync::mpsc;
use tokio::sync::watch;

const MIB: u64 = 1024 * 1024;
const DEPTH: u16 = 6;

fn mb_2(c: &mut Criterion) {
    c.bench_function("mb_2", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(2 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_4(c: &mut Criterion) {
    c.bench_function("mb_4", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(4 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_8(c: &mut Criterion) {
    c.bench_function("mb_8", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(8 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_16(c: &mut Criterion) {
    c.bench_function("mb_16", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(16 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_32(c: &mut Criterion) {
    c.bench_function("mb_32", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(32 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_64(c: &mut Criterion) {
    c.bench_function("mb_64", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(64 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_128(c: &mut Criterion) {
    c.bench_function("mb_128", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(128 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

fn mb_256(c: &mut Criterion) {
    c.bench_function("mb_256", |b| {
        b.to_async(tokio::runtime::Builder::new_multi_thread().build().unwrap())
            .iter(|| async {
                let pos = Position::default();
                let history = PositionHashHistory::new(pos.hash());
                let (_c_tx, c_rx) = watch::channel(());
                let mut tt = TranspositionTable::new(256 * MIB);
                let stats = StatisticsHolder::new();
                let mut searcher = Searcher::with_evaluator_and_config(
                    black_box(history),
                    black_box(pos),
                    c_rx,
                    MainEvaluator::new(),
                    SearcherConfig { depth: Some(DEPTH) },
                    &stats,
                    &mut tt,
                );
                let (tx, _rx) = mpsc::unbounded_channel();
                searcher.search(tx);
            });
    });
}

criterion_group! {
    name = search;
    config = Criterion::default();
    targets = mb_2, mb_4, mb_8, mb_16, mb_32, mb_64, mb_128, mb_256
}
criterion_main!(search);
