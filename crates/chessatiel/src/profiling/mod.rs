use guts::{MoveGenerator, Position};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ProfileMode {
    PerftDefault7,
    PerftOneOfEach6,
}

pub fn run_profile(profile_mode: ProfileMode) {
    match profile_mode {
        ProfileMode::PerftDefault7 => perft_default_7(),
        ProfileMode::PerftOneOfEach6 => perft_one_of_each_6(),
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
