use guts::{MoveGenerator, Position};
use std::collections::HashMap;
use itertools::Itertools;
use std::str::FromStr;

#[test]
fn test_divide_3() {
    let generator = MoveGenerator::new();

    let position = Position::default();

    let divided = divide(&generator, &position, 3);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a2a3: 380
b2b3: 420
c2c3: 420
d2d3: 539
e2e3: 599
f2f3: 380
g2g3: 420
h2h3: 380
a2a4: 420
b2b4: 421
c2c4: 441
d2d4: 560
e2e4: 600
f2f4: 401
g2g4: 421
h2h4: 420
b1a3: 400
b1c3: 440
g1f3: 440
g1h3: 400".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_2_b1a3() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 2);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a7a6: 20
b7b6: 20
c7c6: 20
d7d6: 20
e7e6: 20
f7f6: 20
g7g6: 20
h7h6: 20
a7a5: 20
b7b5: 20
c7c5: 20
d7d5: 20
e7e5: 20
f7f5: 20
g7g5: 20
h7h5: 20
b8a6: 20
b8c6: 20
g8f6: 20
g8h6: 20".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_1_b1a3_a7a5() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/1ppppppp/8/p7/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 1);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "b2b3: 1
c2c3: 1
d2d3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
b2b4: 1
c2c4: 1
d2d4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
g1f3: 1
g1h3: 1
a3b1: 1
a3c4: 1
a3b5: 1
a1b1: 1".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_1_b2b3_d7d5() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/ppp1pppp/8/3p4/8/1P6/P1PPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 1);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a2a3: 1
c2c3: 1
d2d3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
b3b4: 1
a2a4: 1
c2c4: 1
d2d4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
c1b2: 1
c1a3: 1".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_2_b2b3() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/pppppppp/8/8/8/1P6/P1PPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 2);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a7a6: 21
b7b6: 21
c7c6: 21
d7d6: 21
e7e6: 21
f7f6: 21
g7g6: 21
h7h6: 21
a7a5: 21
b7b5: 21
c7c5: 21
d7d5: 21
e7e5: 21
f7f5: 21
g7g5: 21
h7h5: 21
b8a6: 21
b8c6: 21
g8f6: 21
g8h6: 21".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_2_d2d3() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 2);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a7a6: 27
b7b6: 27
c7c6: 27
d7d6: 27
e7e6: 27
f7f6: 27
g7g6: 27
h7h6: 27
a7a5: 27
b7b5: 27
c7c5: 27
d7d5: 27
e7e5: 27
f7f5: 27
g7g5: 26
h7h5: 27
b8a6: 27
b8c6: 27
g8f6: 27
g8h6: 27".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

#[test]
fn test_divide_1_d2d3_a7a5() {
    let generator = MoveGenerator::new();

    let position = Position::from_str("rnbqkbnr/1ppppppp/8/p7/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let divided = divide(&generator, &position, 1);
    let mut vec = divided.into_iter().collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = "a2a3: 1
b2b3: 1
c2c3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
d3d4: 1
a2a4: 1
b2b4: 1
c2c4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1d2: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
c1d2: 1
c1e3: 1
c1f4: 1
c1g5: 1
c1h6: 1
d1d2: 1
e1d2: 1".split("\n").map(|s| {
        let split = s.split(": ").collect_vec();
        (split[0].to_owned(), split[1].to_owned())
    }).collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));


    assert_eq!(vec, expected);
}

fn divide(generator: &MoveGenerator, position: &Position, depth: usize) -> HashMap<String, String> {
    let mut map = HashMap::new();

    let moves = generator.generate_legal_moves_for(&position);
    for m in moves {
        let mut position = position.clone();
        position.make_move(&m.core_move);
        let res = perft(generator, &position, depth - 1);
        map.insert(m.core_move.to_string(), res.to_string());
    }

    map
}

fn perft(generator: &MoveGenerator, position: &Position, depth: usize) -> usize {
    if depth == 0 {
        1
    } else {
        let moves = generator.generate_legal_moves_for(&position);
        moves.into_iter().fold(0, |acc, m| {
            let mut position = position.clone();
            position.make_move(&m.core_move);
            acc + perft(generator, &position, depth - 1)
        })
    }
}
