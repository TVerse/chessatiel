use criterion::{black_box, criterion_group, criterion_main, Criterion};
use guts::MoveGenerator;
use guts::Position;
use std::str::FromStr;
use std::time::Duration;

fn perft_kiwipete_4(c: &mut Criterion) {
    let movegen = MoveGenerator::new();
    let pos =
        Position::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    c.bench_function("perft_kiwipete_4", |b| {
        b.iter(|| movegen.perft(black_box(&pos), black_box(4)))
    });
}

criterion_group! {
    name = perft;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = perft_kiwipete_4
}
criterion_main!(perft);
