use guts::Position;
use rand::prelude::*;

pub fn generate_tournament_openings(positions: &mut [Position], number: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(std::f64::consts::E.to_bits());
    positions.shuffle(&mut rng);

    for p in positions {
        if result.len() >= number {
            break;
        }
        if p.board().all_pieces().count_ones() > 10 && !result.contains(p) {
            result.push(p.clone())
        }
    }

    result.into_iter().map(|p| p.to_epd()).collect()
}
