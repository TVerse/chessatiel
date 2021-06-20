use brain::Engine;
use guts::Position;
use std::str::FromStr;

#[test]
fn mate_in_one() {
    let engine = Engine::new();
    let position = Position::from_str("8/8/8/8/7k/8/5R2/K5R1 w - - 0 1").unwrap();
    let expected = "f2h2";

    let result = engine.find_move(3, &position);

    assert_eq!(result.unwrap().as_uci(), expected)
}

#[test]
fn mate_in_two() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/8/8/5R2/K3R3 w - - 0 1").unwrap();
    let expected = vec!["f2g2", "e1g1"];

    let result = engine.find_move(3, &position);

    assert!(expected.contains(&result.unwrap().as_uci().as_str()))
}

#[test]
#[ignore]
fn mate_in_four() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/4K3/8/5Q2/8 w - - 0 1").unwrap();

    // perft: 69217046, 3 seconds
    let expected = "f2g3";

    let result = engine.find_move(8, &position);

    assert_eq!(result.unwrap().as_uci(), expected)
}
