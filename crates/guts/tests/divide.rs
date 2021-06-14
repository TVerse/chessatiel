use guts::{MoveGenerator, Position};
use itertools::Itertools;
use std::str::FromStr;

#[test]
fn test_divide_3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        3,
        "a2a3: 380
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
g1h3: 400",
    );
}

#[test]
fn test_divide_2_b1a3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 0 1",
        2,
        "a7a6: 20
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
g8h6: 20",
    );
}

#[test]
fn test_divide_1_b1a3_a7a5() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
        1,
        "b2b3: 1
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
a1b1: 1",
    );
}

#[test]
fn test_divide_1_b2b3_d7d5() {
    test_divide(
        "rnbqkbnr/ppp1pppp/8/3p4/8/1P6/P1PPPPPP/RNBQKBNR w KQkq - 0 1",
        1,
        "a2a3: 1
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
c1a3: 1",
    );
}

#[test]
fn test_divide_2_b2b3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/1P6/P1PPPPPP/RNBQKBNR b KQkq - 0 1",
        2,
        "a7a6: 21
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
g8h6: 21",
    )
}

#[test]
fn test_divide_2_d2d3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1",
        2,
        "a7a6: 27
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
g8h6: 27",
    );
}

#[test]
fn test_divide_1_d2d3_a7a5() {
    test_divide(
        "rnbqkbnr/1ppppppp/8/p7/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1",
        1,
        "a2a3: 1
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
e1d2: 1",
    );
}

#[test]
fn test_divide_4() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        4,
        "a2a3: 8457
b2b3: 9345
c2c3: 9272
d2d3: 11959
e2e3: 13134
f2f3: 8457
g2g3: 9345
h2h3: 8457
a2a4: 9329
b2b4: 9332
c2c4: 9744
d2d4: 12435
e2e4: 13160
f2f4: 8929
g2g4: 9328
h2h4: 9329
b1a3: 8885
b1c3: 9755
g1f3: 9748
g1h3: 8881",
    )
}

#[test]
fn test_divide_3_b1a3() {
    test_divide(
        "rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
        3,
        "b2b3: 400
c2c3: 460
d2d3: 519
e2e3: 599
f2f3: 380
g2g3: 420
h2h3: 380
b2b4: 401
c2c4: 441
d2d4: 540
e2e4: 600
f2f4: 401
g2g4: 421
h2h4: 420
g1f3: 440
g1h3: 400
a3b1: 400
a3c4: 480
a3b5: 475
a1b1: 380",
    )
}

#[test]
fn test_kiwipete_3() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        3,
        "a2a3: 2186
b2b3: 1964
g2g3: 1882
d5d6: 1991
a2a4: 2149
g2g4: 1843
g2h3: 1970
d5e6: 2241
c3b1: 2038
c3d1: 2040
c3a4: 2203
c3b5: 2138
e5d3: 1803
e5c4: 1880
e5g4: 1878
e5c6: 2027
e5g6: 1997
e5d7: 2124
e5f7: 2080
d2c1: 1963
d2e3: 2136
d2f4: 2000
d2g5: 2134
d2h6: 2019
e2d1: 1733
e2f1: 2060
e2d3: 2050
e2c4: 2082
e2b5: 2057
e2a6: 1907
a1b1: 1969
a1c1: 1968
a1d1: 1885
h1f1: 1929
h1g1: 2013
f3d3: 2005
f3e3: 2174
f3g3: 2214
f3h3: 2360
f3f4: 2132
f3g4: 2169
f3f5: 2396
f3h5: 2267
f3f6: 2111
e1d1: 1894
e1f1: 1855
e1g1: 2059
e1c1: 1887",
    )
}

#[test]
fn test_kiwipete_2_e1f1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R4K1R b kq - 0 1",
        2,
        "b4b3: 46
g6g5: 44
c7c6: 46
d7d6: 44
c7c5: 46
h3g2: 4
e6d5: 45
b4c3: 45
b6a4: 44
b6c4: 44
b6d5: 45
b6c8: 45
f6e4: 48
f6g4: 44
f6d5: 46
f6h5: 46
f6h7: 46
f6g8: 46
a6e2: 5
a6d3: 43
a6c4: 43
a6b5: 44
a6b7: 46
a6c8: 46
g7h6: 45
g7f8: 45
a8b8: 45
a8c8: 45
a8d8: 45
h8h4: 45
h8h5: 45
h8h6: 45
h8h7: 45
h8f8: 45
h8g8: 45
e7c5: 45
e7d6: 44
e7d8: 45
e7f8: 45
e8d8: 45
e8f8: 45
e8g8: 45
e8c8: 45",
    )
}

#[test]
fn test_kiwipete_1_e1f1_a6e2() {
    test_divide(
        "r3k2r/p1ppqpb1/1n2Pnp1/4N3/1p2P3/2N2Q1p/PPPBbPPP/R4K1R w kq - 0 1",
        1,
        "f1e1: 1
f1g1: 1
f1e2: 1
c3e2: 1
f3e2: 1",
    )
}

#[test]
fn test_kiwipete_1_e1f1_h3g2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/R4K1R w kq - 0 1",
        1,
        "f1e1: 1
f1g1: 1
f1g2: 1
f3g2: 1",
    )
}

#[test]
fn test_kiwipete_2_a1b1() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBBPPP/R3K2R b KQkq - 0 1",
        2,
        "b4b3: 49
g6g5: 49
c7c6: 51
d7d6: 49
c7c5: 51
h3g2: 48
b4a3: 51
e6d5: 50
b4c3: 48
b6a4: 49
b6c4: 48
b6d5: 50
b6c8: 50
f6e4: 53
f6g4: 49
f6d5: 51
f6h5: 51
f6h7: 51
f6g8: 51
a6e2: 43
a6d3: 48
a6c4: 48
a6b5: 49
a6b7: 50
a6c8: 50
g7h6: 50
g7f8: 50
a8b8: 50
a8c8: 50
a8d8: 50
h8h4: 50
h8h5: 50
h8h6: 50
h8h7: 50
h8f8: 50
h8g8: 50
e7c5: 50
e7d6: 49
e7d8: 50
e7f8: 50
e8d8: 50
e8f8: 50
e8g8: 50
e8c8: 50",
    )
}

#[test]
fn test_kiwipete_1_a1b1_h3g2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/R3K2R w KQkq - 0 1",
        1,
        "b2b3: 1
h2h3: 1
a3a4: 1
d5d6: 1
h2h4: 1
a3b4: 1
d5e6: 1
c3b1: 1
c3d1: 1
c3a2: 1
c3a4: 1
c3b5: 1
e5d3: 1
e5c4: 1
e5g4: 1
e5c6: 1
e5g6: 1
e5d7: 1
e5f7: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2c4: 1
e2b5: 1
e2a6: 1
a1b1: 1
a1c1: 1
a1d1: 1
a1a2: 1
h1f1: 1
h1g1: 1
f3g2: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3h3: 1
f3f4: 1
f3g4: 1
f3f5: 1
f3h5: 1
f3f6: 1
e1d1: 1
e1c1: 1",
    )
}

#[test]
fn test_kiwipete_1_a1b1_a6e2() {
    test_divide(
        "r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/P1N2Q1p/1PPBbPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
a7a6: 1
c7c6: 1
d7d6: 1
a7a5: 1
c7c5: 1
h3g2: 1
b4a3: 1
e6d5: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2f3: 1
e2c4: 1
e2b5: 1
e2a6: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_2() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        2,
        "a2a3: 44
b2b3: 42
g2g3: 42
d5d6: 41
a2a4: 44
g2g4: 42
g2h3: 43
d5e6: 46
c3b1: 42
c3d1: 42
c3a4: 42
c3b5: 39
e5d3: 43
e5c4: 42
e5g4: 44
e5c6: 41
e5g6: 42
e5d7: 45
e5f7: 44
d2c1: 43
d2e3: 43
d2f4: 43
d2g5: 42
d2h6: 41
e2d1: 44
e2f1: 44
e2d3: 42
e2c4: 41
e2b5: 39
e2a6: 36
a1b1: 43
a1c1: 43
a1d1: 43
h1f1: 43
h1g1: 43
f3d3: 42
f3e3: 43
f3g3: 43
f3h3: 43
f3f4: 43
f3g4: 43
f3f5: 45
f3h5: 43
f3f6: 39
e1d1: 43
e1f1: 43
e1g1: 43
e1c1: 43",
    )
}

#[test]
fn test_kiwipete_1_f3h3() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N4Q/PPPBBPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
c7c6: 1
d7d6: 1
c7c5: 1
e6d5: 1
b4c3: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h3: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_kiwipete_1_d5e6() {
    test_divide(
        "r3k2r/p1ppqpb1/bn2Pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
        1,
        "b4b3: 1
g6g5: 1
c7c6: 1
d7d6: 1
c7c5: 1
d7d5: 1
h3g2: 1
f7e6: 1
b4c3: 1
d7e6: 1
b6a4: 1
b6c4: 1
b6d5: 1
b6c8: 1
f6e4: 1
f6g4: 1
f6d5: 1
f6h5: 1
f6h7: 1
f6g8: 1
a6e2: 1
a6d3: 1
a6c4: 1
a6b5: 1
a6b7: 1
a6c8: 1
g7h6: 1
g7f8: 1
a8b8: 1
a8c8: 1
a8d8: 1
h8h4: 1
h8h5: 1
h8h6: 1
h8h7: 1
h8f8: 1
h8g8: 1
e7c5: 1
e7d6: 1
e7e6: 1
e7d8: 1
e7f8: 1
e8d8: 1
e8f8: 1
e8g8: 1
e8c8: 1",
    )
}

#[test]
fn test_case_1() {
    test_divide(
        "4k3/3pqp2/4P3/8/8/8/8/4K3 w - - 0 1",
        2,
        "e1f2: 18
e1e2: 18
e1d1: 18
e1f1: 18
e1d2: 18",
    )
}

fn test_divide(position: &str, depth: usize, expected: &str) {
    let generator = MoveGenerator::new();

    let position = Position::from_str(position).unwrap();

    let divided = generator.divide(&position, depth);
    let mut vec = divided
        .into_iter()
        .map(|(m, c)| {
            (
                format!("{}{}", m.from.to_string(), m.to.to_string()),
                c.to_string(),
            )
        })
        .collect_vec();
    vec.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    let mut expected = expected
        .split("\n")
        .map(|s| {
            let split = s.split(": ").collect_vec();
            (split[0].to_owned(), split[1].to_owned())
        })
        .collect_vec();
    expected.sort_by(|d1, d2| d1.0.cmp(&d2.0));

    assert_eq!(vec, expected);
}
