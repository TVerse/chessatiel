use guts::{Color, Piece, Position, Square};

const TABLE_SIZE: usize = 2 * 6 * 64; // mid/end * pieces * squares

#[derive(Debug)]
pub struct PieceSquareTable {
    values: Vec<f32>,
}

impl PieceSquareTable {
    pub fn zeroes() -> Self {
        let mut values = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            values.push(0.0)
        }
        Self { values }
    }

    pub fn values_mut(&mut self) -> &mut [f32] {
        &mut self.values
    }

    pub fn get(&self, position: &Position) -> f32 {
        // White is positive, these tables are not current-relative
        let sgn = if position.active_color() == Color::White {
            1.0
        } else {
            -1.0
        };
        sgn * dot(&Self::position_as_vec(position), &self.values)
    }

    pub fn position_as_vec(position: &Position) -> Vec<(usize, f32)> {
        let endgame_factor = Self::endgame_factor(position);
        let mut vec = Vec::with_capacity(position.board().all_pieces().count_ones() as usize);

        for p in Piece::ALL {
            let white = position.board()[Color::White][p].squares();
            for s in white {
                let (midgame_idx, endgame_idx) = Self::indices_for(p, s);
                vec.push((midgame_idx, 1.0 - endgame_factor));
                vec.push((endgame_idx, endgame_factor));
            }
            let black = position.board()[Color::Black][p].squares();
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

    pub fn endgame_factor(position: &Position) -> f32 {
        1.0 - ((position.board().all_pieces().count_ones() as f32 - 2.0) / 38.0)
    }

    pub fn from_json_str(json: &str) -> Self {
        let values = serde_json::from_str(json).expect("JSON for PST was invalid");
        Self { values }
    }
}

pub fn dot(a: &[(usize, f32)], b: &[f32]) -> f32 {
    let mut product = 0.0;

    for (idx, a) in a {
        product += a * b[*idx]
    }

    product
}
