use chessatiel::brain::Engine;
use guts::Position;
use std::str::FromStr;
use std::sync::atomic;

fn assert_nodes_searched(engine: &Engine, expected: u64) {
    let nodes_searched = engine
        .statistics()
        .nodes_searched()
        .load(atomic::Ordering::Acquire);
    assert_eq!(
        nodes_searched, expected,
        "Unexpected number of nodes searched: got {}, expected {}",
        nodes_searched, expected
    );
}

#[test]
fn mate_in_one() {
    let mut engine = Engine::new();
    let position = Position::from_str("8/8/8/8/7k/8/5R2/K5R1 w - - 0 1").unwrap();
    let expected = "f2h2";
    let depth_nodes = [(2, 60), (3, 909), (4, 1489), (5, 16079), (6, 25046)];

    for (depth, expected_nodes) in depth_nodes {
        engine.reset_tables();
        let result = engine.search(depth, &position);

        assert_eq!(
            result.clone().map(|sr| sr.chess_move().as_uci()),
            Some(expected.to_string()),
            "Wrong answer for depth {}: {:?}",
            depth,
            &result
        );
        assert_nodes_searched(&engine, expected_nodes);
    }
}

#[test]
fn mate_in_two() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/8/8/5R2/K3R3 w - - 0 1").unwrap();
    let expected = vec!["f2g2", "e1g1"];

    let result = engine.search(4, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
    assert_nodes_searched(&engine, 1505);
}

#[test]
fn mate_in_two_2() {
    let engine = Engine::new();
    let position =
        Position::from_str("8/4k3/Knbppp2/4q1b1/4p2n/3p1p2/3p1p2/3r1r2 b - - 0 1").unwrap();
    let expected = vec!["b6d7", "e5b5"];

    let result = engine.search(4, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
    assert_nodes_searched(&engine, 4612);
}

#[test]
fn mate_in_four() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/4K3/8/5Q2/8 w - - 0 1").unwrap();

    // perft: 69217046, 3 seconds
    let expected = "e4f5";

    let result = engine.search(8, &position);

    assert_eq!(result.unwrap().chess_move().as_uci(), expected);
    assert_nodes_searched(&engine, 435643);
}

#[test]
#[ignore]
fn mate_in_five() {
    let engine = Engine::new();
    let position =
        Position::from_str("4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1").unwrap();

    let expected = "d1a4";

    let result = engine.search(8, &position);

    assert_eq!(result.unwrap().chess_move().as_uci(), expected);
    assert_nodes_searched(&engine, 4756012);
}
