use chessatiel::brain::Engine;
use guts::Position;
use std::str::FromStr;

// TODO move this to some benchmark/perf test, with default engine it's machine-specific
// fn assert_max_nodes_searched(engine: &Engine, expected: u64) {
//     let nodes_searched = engine
//         .statistics()
//         .nodes_searched()
//         .load(atomic::Ordering::Acquire);
//     assert!(
//         nodes_searched <= expected,
//         "Unexpected number of nodes searched: got {}, expected at most {}",
//         nodes_searched,
//         expected
//     );
// }

#[test]
fn mate_in_one() {
    let mut engine = Engine::new();
    let position = Position::from_str("8/8/8/8/7k/8/5R2/K5R1 w - - 0 1").unwrap();
    let expected = "f2h2";
    let depth_nodes = [(2, 60), (3, 909), (4, 1489), (5, 17228), (6, 25174)];

    for (depth, _expected_nodes) in depth_nodes {
        engine.reset_tables();
        let result = engine.search(depth, &position);

        assert_eq!(
            result.clone().map(|sr| sr.chess_move().as_uci()),
            Some(expected.to_string()),
            "Wrong answer for depth {}: {:?}",
            depth,
            &result
        );
        // assert_max_nodes_searched(&engine, expected_nodes);
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
    // assert_max_nodes_searched(&engine, 1505);
}

#[test]
fn mate_in_two_2() {
    let engine = Engine::new();
    let position =
        Position::from_str("8/4k3/Knbppp2/4q1b1/4p2n/3p1p2/3p1p2/3r1r2 b - - 0 1").unwrap();
    let expected = vec!["b6d7", "e5b5"];

    let result = engine.search(6, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
    // assert_max_nodes_searched(&engine, 4614);
}

#[test]
fn mate_in_four() {
    let engine = Engine::new();
    let position = Position::from_str("8/7k/8/8/4K3/8/5Q2/8 w - - 0 1").unwrap();

    let expected = vec!["f2g3", "f2g1", "f2g2", "e4f5"];

    let result = engine.search(10, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
    // assert_max_nodes_searched(&engine, 459550);
}

#[test]
#[ignore]
fn mate_in_five() {
    let engine = Engine::new();
    let position =
        Position::from_str("4r3/7q/nb2prRp/pk1p3P/3P4/P7/1P2N1P1/1K1B1N2 w - - 0 1").unwrap();

    let expected = "d1a4";

    let result = engine.search(10, &position);

    assert_eq!(result.unwrap().chess_move().as_uci(), expected);
    // assert_max_nodes_searched(&engine, 144308171);
}

#[test]
fn promotion_mate() {
    let engine = Engine::new();
    let position = Position::from_str("8/2P5/8/8/8/8/Q7/2k1K3 w - - 0 1").unwrap();

    let expected = vec!["c7c8q", "c7c8r"];

    let result = engine.search(4, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
}

#[test]
fn castle_mate() {
    let engine = Engine::new();
    let position = Position::from_str("5k2/Q6B/2B5/8/8/8/8/4K2R w K - 0 1").unwrap();

    let expected = vec!["e1g1", "h1f1"];

    let result = engine.search(4, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
}

#[test]
fn en_passant_mate() {
    let engine = Engine::new();
    let position = Position::from_str("8/7B/7B/5pP1/8/8/Q3PP2/2k1K3 w - f6 0 1").unwrap();

    let expected = vec!["g5f6"];

    let result = engine.search(4, &position);

    assert!(
        expected.contains(&result.clone().unwrap().chess_move().as_uci().as_str()),
        "{:?} did not contain {:?}",
        expected,
        &result.unwrap().chess_move().as_uci()
    );
}
