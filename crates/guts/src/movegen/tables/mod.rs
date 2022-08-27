use crate::bitboard::Bitboard;
use crate::square::Square;
pub use knight::KnightMovePatterns;
pub use squares_between::SquaresBetween;

mod knight;
mod squares_between;

struct GenerateInput {
    dr: i16,
    df: i16,
}

fn generate<P: Fn(GenerateInput) -> bool>(p: P) -> [Bitboard; 64] {
    let mut map = [Bitboard::EMPTY; 64];
    for from in Square::ALL.iter() {
        let from_rank = from.rank().index() as i16;
        let from_file = from.file().index() as i16;
        let to = Square::ALL.iter().filter(|&to| {
            let to_rank = to.rank().index() as i16;
            let to_file = to.file().index() as i16;

            let dr = to_rank - from_rank;
            let df = to_file - from_file;

            let gi = GenerateInput { dr, df };

            (from != to) && p(gi)
        });
        let bb = Bitboard::from_iter(to.copied());
        map[from.bitboard_index()] = bb
    }
    map
}
