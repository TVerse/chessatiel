use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::MoveGenerator;
use guts::Position;
use std::str::FromStr;

fn pawns_move(c: &mut Criterion) {
    let movegen = MoveGenerator::new();
    let mut pos = Position::from_str("k7/ppppp3/5p2/P5p1/1P5p/2P5/3PPPPP/K7 w - - 0 1").unwrap();
    c.bench_function("pawns_move", |b| {
        b.iter(|| movegen.perft(black_box(&mut pos), black_box(5)))
    });
}

criterion_group! {
    name = pawns;
    config = Criterion::default();
    targets = pawns_move
}
criterion_main!(pawns);
