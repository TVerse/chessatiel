use clap::Clap;
use guts::{MoveGenerator, Position};
use std::str::FromStr;

#[derive(Clap, Debug)]
#[clap(name = "chessatiel")]
struct CliOpts {
    position: String,
    depth: usize,
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let movegen = MoveGenerator::new();
    let position = Position::from_str(&opts.position).unwrap();
    println!("Running...");
    movegen.perft(&position, opts.depth);
    println!("Done!");
}
