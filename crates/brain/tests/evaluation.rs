use brain::Engine;
use guts::Position;
use std::str::FromStr;
use std::sync::atomic;

fn assert_nodes_searched(engine: &Engine, expected: u64) {
    let nodes_searched = engine.statistics().nodes_searched().load(atomic::Ordering::Acquire);
    assert_eq!(nodes_searched, expected, "Unexpected number of nodes searched: got {}, expected {}", nodes_searched, expected);
}

#[test]
fn mate_in_one() {
    let engine = Engine::new();
    let position = Position::from_str("8/8/8/8/7k/8/5R2/K5R1 w - - 0 1").unwrap();
    let expected = "f2h2";
    let depth_nodes = [
        (2, 60),
        (3, 909),
        (4, 1787),
        (5, 26439),
        (6, 50912),
    ];

    for (depth, expected_nodes) in depth_nodes {
        let result = engine.search(depth, &position);

        assert_eq!(result.clone().map(|sr| sr.chess_move().as_uci()), Some(expected.to_string()), "Wrong answer for depth {}: {:?}", depth, &result);
        let nodes_searched = engine.statistics().nodes_searched().load(atomic::Ordering::Acquire);
        println!("{} {}", depth, nodes_searched);
        assert_nodes_searched(&engine, expected_nodes);
    }
}

#[test]
fn mate_in_two() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/8/8/5R2/K3R3 w - - 0 1").unwrap();
    let expected = vec!["f2g2", "e1g1"];

    let result = engine.search(4, &position);

    assert!(expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()), "{:?} did not contain {:?}", expected, &result.unwrap().chess_move().as_uci());
    assert_nodes_searched(&engine, 1890);
}

#[test]
fn mate_in_four() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/4K3/8/5Q2/8 w - - 0 1").unwrap();

    // perft: 69217046, 3 seconds
    let expected = "e4f5";

    let result = engine.search(8, &position);

    assert_eq!(result.unwrap().chess_move().as_uci(), expected);
    assert_nodes_searched(&engine, 1471144);
}
