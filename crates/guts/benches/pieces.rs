// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use guts::MoveGenerator;
// use guts::Position;
// use std::str::FromStr;
// use std::time::Duration;
//
// fn bench_pawns_move(c: &mut Criterion) {
//     let movegen = MoveGenerator::new();
//     let pos = Position::from_str("k7/ppppp3/5p2/P5p1/1P5p/2P5/3PPPPP/K7 w - - 0 1").unwrap();
//     c.bench_function("pawns move", |b| {
//         b.iter(|| movegen.perft(black_box(&pos), black_box(5)))
//     });
// }
//
// criterion_group!(
//     name = pawns;
//     config = Criterion::default().measurement_time(Duration::from_secs(30));
//     targets = bench_pawns_move);
// criterion_main!(pawns);
