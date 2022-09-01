use guts::{Color, Piece, Position, Square};

const TABLE_SIZE: usize = 2 * 6 * 64; // mid/end * pieces * squares

#[derive(Debug)]
pub struct PieceSquareTable {
    values: Vec<f64>,
}

impl PieceSquareTable {
    pub fn zeroes() -> Self {
        let mut values = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            values.push(0.0)
        }
        Self { values }
    }

    pub fn piece_values() -> Self {
        let mut pst = Self::zeroes();
        for p in Piece::ALL {
            let value = match p {
                Piece::Pawn => 1.0,
                Piece::Knight => 3.0,
                Piece::Bishop => 3.0,
                Piece::Rook => 5.0,
                Piece::Queen => 9.0,
                Piece::King => 4.5,
            };
            for s in Square::ALL {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                pst.values[midgame_idx] = value;
                pst.values[endgame_idx] = value;
            }
        }
        pst
    }

    pub fn values_mut(&mut self) -> &mut [f64] {
        &mut self.values
    }

    pub fn get(&self, position: &Position) -> f64 {
        // White is positive, these tables are not current-relative
        let sgn = if position.active_color() == Color::White {
            1.0
        } else {
            -1.0
        };
        let mut res = 0.0;
        let endgame_factor = Self::endgame_factor(position);
        // TODO code duplication
        for p in Piece::ALL {
            let white = position.board()[Color::White][p].into_iter();
            for s in white {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                res += self.values[midgame_idx] * (1.0 - endgame_factor);
                res += self.values[endgame_idx] * endgame_factor;
            }
            let black = position.board()[Color::Black][p].into_iter();
            for s in black {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                res -= self.values[midgame_idx] * (1.0 - endgame_factor);
                res -= self.values[endgame_idx] * endgame_factor;
            }
        }
        sgn * res
    }

    pub fn position_as_vec(position: &Position) -> Vec<(usize, f64)> {
        let endgame_factor = Self::endgame_factor(position);
        let mut vec = Vec::with_capacity(position.board().all_pieces().count_ones() as usize);

        for p in Piece::ALL {
            let white = position.board()[Color::White][p].into_iter();
            for s in white {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                vec.push((midgame_idx, 1.0 - endgame_factor));
                vec.push((endgame_idx, endgame_factor));
            }
            let black = position.board()[Color::Black][p].into_iter();
            for s in black {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                vec.push((midgame_idx, -1.0 + endgame_factor));
                vec.push((endgame_idx, -endgame_factor));
            }
        }

        vec
    }

    fn indices_for(p: Piece, s: Square) -> (usize, usize) {
        // TODO best ordering?
        let midgame_idx = 2 * (s.bitboard_index() * 6 + p.index());
        let endgame_idx = midgame_idx + 1;
        assert!(endgame_idx < TABLE_SIZE);
        (midgame_idx, endgame_idx)
    }

    pub fn endgame_factor(position: &Position) -> f64 {
        1.0 - ((position.board().all_pieces().count_ones() as f64 - 2.0) / 38.0)
    }

    pub fn from_bincode(data: &[u8]) -> Self {
        let values = bincode::deserialize(data).expect("Bincode for PST was invalid");
        Self { values }
    }
}

pub fn dot(a: &[(usize, f64)], b: &[f64]) -> f64 {
    let mut product = 0.0;

    for (idx, a) in a {
        product += a * b[*idx]
    }

    product
}
