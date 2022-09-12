use brain::position_hash_history::PositionHashHistory;
use brain::searcher::{Searcher, SearcherConfig};
use brain::statistics::StatisticsHolder;
use brain::transposition_table::TranspositionTable;
use guts::{MoveGenerator, Position};
use std::str::FromStr;
use tokio::sync::{mpsc, watch};

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ProfileMode {
    PerftDefault7,
    PerftOneOfEach6,
    SearchDepth,
}

pub async fn run_profile(profile_mode: ProfileMode) {
    match profile_mode {
        ProfileMode::PerftDefault7 => perft_default_7(),
        ProfileMode::PerftOneOfEach6 => perft_one_of_each_6(),
        ProfileMode::SearchDepth => search_depth(9).await,
    }
}

fn perft_default_7() {
    let mut position = Position::default();
    let move_generator = MoveGenerator::new();

    let res = move_generator.perft(&mut position, 7);
    println!("{}", res)
}

fn perft_one_of_each_6() {
    let mut position = Position::from_str("rnbqkbnr/7p/8/8/8/8/P7/RNBQKBNR w KQkq - 0 1").unwrap();
    let move_generator = MoveGenerator::new();

    let res = move_generator.perft(&mut position, 6);
    println!("{}", res)
}

async fn search_depth(depth: u16) {
    let position = Position::default();
    let history = PositionHashHistory::new(position.hash());
    let mut tt = TranspositionTable::default();
    let stats = StatisticsHolder::new();
    let (_stop_tx, stop_rx) = watch::channel(());
    let config = SearcherConfig { depth: Some(depth) };
    let (result_tx, mut result_rx) = mpsc::unbounded_channel();
    let _search_task = std::thread::spawn(move || {
        let mut searcher = Searcher::new(history, position, stop_rx, config, &stats, &mut tt);
        searcher.search(result_tx);
    });
    let mut res = None;
    while let Some(r) = result_rx.recv().await {
        res = Some(r);
    }
    println!("{:?}", res)
}
