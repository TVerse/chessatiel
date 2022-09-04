use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::MoveGenerator;
use guts::Position;
use std::str::FromStr;

fn unmake(c: &mut Criterion) {
    let movegen = MoveGenerator::new();
    let mut pos =
        Position::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    c.bench_function("unmake", |b| {
        b.iter(|| movegen.perft(black_box(&mut pos), black_box(4)))
    });
}

fn clone(c: &mut Criterion) {
    let movegen = MoveGenerator::new();
    let pos =
        Position::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    c.bench_function("clone", |b| {
        b.iter(|| movegen.perft_clone(black_box(&pos), black_box(4)))
    });
}

criterion_group! {
    name = unmake_vs_clone;
    config = Criterion::default();
    targets = unmake, clone
}
criterion_main!(unmake_vs_clone);
