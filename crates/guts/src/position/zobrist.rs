use crate::board::Board;
use crate::file::File;
use crate::position::State;
use crate::square::Square;
use crate::{Color, Piece};
use once_cell::sync::Lazy;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    pub fn flip_piece(&mut self, color: Color, piece: Piece, square: Square) {
        self.0 ^= Zobrist::get().for_tuple(color, piece, square)
    }

    pub fn flip_ep_file(&mut self, file: File) {
        self.0 ^= Zobrist::get().ep_file[file.index()]
    }

    pub fn flip_castle_rights(&mut self, color: Color, kingside: bool) {
        self.0 ^= Zobrist::get().castling_rights[color.index()][if kingside { 0 } else { 1 }]
    }

    pub fn flip_side_to_move(&mut self) {
        self.0 ^= Zobrist::get().side_to_move_is_black
    }
}

static ZOBRIST: Lazy<Zobrist> = Lazy::new(|| Zobrist::generate(std::f64::consts::E.to_bits()));

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

    fn for_tuple(&self, color: Color, piece: Piece, square: Square) -> u64 {
        self.pieces[color.index()][piece.index()][square.bitboard_index()]
    }

    pub fn for_position(&self, board: &Board, state: &State) -> ZobristHash {
        let mut hash = 0x0;
        for s in Square::ALL {
            if let Some((p, c)) = board.piece_and_color_at(s) {
                hash ^= self.for_tuple(c, p, s)
            }
        }

        if state.active_color == Color::Black {
            hash ^= self.side_to_move_is_black;
        }

        for c in Color::ALL {
            if state.castle_rights[c].kingside {
                hash ^= self.castling_rights[c.index()][0];
            }
            if state.castle_rights[c].queenside {
                hash ^= self.castling_rights[c.index()][1];
            }
        }

        if let Some(s) = state.en_passant {
            let file = s.file().index();
            hash ^= self.ep_file[file];
        }

        ZobristHash(hash)
    }

    #[cfg(test)]
    #[allow(unused)]
    pub fn find_hash(&self, hash: ZobristHash) -> String {
        let hash = hash.0;
        if self.side_to_move_is_black == hash {
            return "side_to_move_is_black".to_string();
        }

        for c in Color::ALL {
            for h in self.castling_rights[c.index()] {
                if h == hash {
                    return format!("Castling: {c} {h}");
                }
            }
        }

        for f in File::ALL {
            if self.ep_file[f.index()] == hash {
                return format!("EP file: {f}");
            }
        }

        for c in Color::ALL {
            for p in Piece::ALL {
                for s in Square::ALL {
                    if self.pieces[c.index()][p.index()][s.bitboard_index()] == hash {
                        return format!("Pieces: {c} {p} {s}");
                    }
                }
            }
        }

        "Not found".to_string()
    }
}
