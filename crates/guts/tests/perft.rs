use guts::{MoveGenerator, Position};
use std::str::FromStr;

#[test]
fn test_perft_movegen_starting_board() {
    let generator = MoveGenerator::new();

    let starting_position = Position::default();

    let count = generator.perft(&starting_position, 1);
    assert_eq!(count, 20);

    let count = generator.perft(&starting_position, 2);
    assert_eq!(count, 400);

    let count = generator.perft(&starting_position, 3);
    assert_eq!(count, 8902);

    let count = generator.perft(&starting_position, 4);
    assert_eq!(count, 197281);
}

#[test]
fn test_kiwipete() {
    let generator = MoveGenerator::new();

    let position =
        Position::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

    let count = generator.perft(&position, 1);
    assert_eq!(count, 48);

    let count = generator.perft(&position, 2);
    assert_eq!(count, 2039);

    let count = generator.perft(&position, 3);
    assert_eq!(count, 97862);
}

#[test]
fn no_moves() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("8/8/8/8/8/1q6/2q5/K7 w - - 0 1").unwrap();

    let count = generator.perft(&position, 1);
    assert_eq!(count, 0);

    let count = generator.perft(&position, 2);
    assert_eq!(count, 0);
}

#[test]
fn one_move() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("8/8/8/8/8/1q6/K1q5/8 w - - 0 1").unwrap();

    let count = generator.perft(&position, 1);
    assert_eq!(count, 1);
}

#[test]
fn test_case_1() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("4k3/3pqp2/4P3/8/8/8/8/4K3 b - - 0 1").unwrap();

    let count = generator.perft(&position, 1);
    assert_eq!(count, 18);
}
