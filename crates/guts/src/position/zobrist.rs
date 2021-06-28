use crate::board::Board;
use crate::file::File;
use crate::position::State;
use crate::square::Square;
use crate::{Color, Piece};
use lazy_static::lazy_static;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ZobristHash(u64);

impl ZobristHash {
    pub const ZERO: ZobristHash = ZobristHash(0);
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for ZobristHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO but probably end up using preallocated exact-fit vec
        self.0.hash(state)
    }
}

lazy_static! {
    static ref ZOBRIST: Zobrist = Zobrist::generate(std::f64::consts::E.to_bits());
}

#[derive(Debug)]
pub struct Zobrist {
    pieces: [[[u64; Square::NUM]; Piece::NUM]; Color::NUM],
    side_to_move_is_black: u64,
    castling_rights: [[u64; 2]; Color::NUM],
    ep_file: [u64; File::NUM],
}

impl Distribution<Zobrist> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Zobrist {
        let mut pieces = [[[0; Square::NUM]; Piece::NUM]; Color::NUM];
        for c in pieces.iter_mut() {
            for p in c.iter_mut() {
                rng.fill(p);
            }
        }
        let mut castling_rights = [[0; 2]; Color::NUM];
        for c in castling_rights.iter_mut() {
            rng.fill(c);
        }
        let mut ep_file = [0; File::NUM];
        rng.fill(&mut ep_file);
        Zobrist {
            pieces,
            side_to_move_is_black: rng.gen(),
            castling_rights,
            ep_file,
        }
    }
}

impl Zobrist {
    pub fn get() -> &'static Self {
        &ZOBRIST
    }

    fn generate(seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);

        rng.gen()
    }

    pub fn for_position(&self, board: &Board, state: &State) -> ZobristHash {
        let mut hash = 0x0;
        for s in Square::ALL {
            if let Some((p, c)) = board.piece_and_color_at(s) {
                hash ^= self.pieces[c.index()][p.index()][s.bitboard_index()]
            }
        }

        if state.active_color == Color::Black {
            hash ^= self.side_to_move_is_black;
        }

        for c in Color::ALL {
            if state.castle_rights[c].kingside {
                hash ^= self.castling_rights[c.index()][0];
            }
            if state.castle_rights[c].kingside {
                hash ^= self.castling_rights[c.index()][1];
            }
        }

        if let Some(s) = state.en_passant {
            let file = s.file().index();
            hash ^= self.ep_file[file];
        }

        ZobristHash(hash)
    }
}
