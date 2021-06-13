use crate::bitboard::Bitboard;
use crate::square::Square;

// TODO figure out how to test correctness.

pub struct SquaresBetween {
    squares_between: [[Bitboard; 64]; 64],
}

impl SquaresBetween {
    pub fn new() -> Self {
        let mut squares_between = [[Bitboard::EMPTY; 64]; 64];

        for s1 in Square::ALL.iter() {
            for s2 in Square::ALL.iter() {
                let bb1 = Bitboard::from_square(*s1);
                let bb2 = Bitboard::from_square(*s2);

                let df = s2.file() as i16 - s1.file() as i16;
                let dr = s2.rank() as i16 - s1.rank() as i16;

                let same_file = df == 0;
                let same_rank = dr == 0;

                if same_file {
                    if dr > 0 {
                        // s2 is above s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.north_attack(!bb2) & !bb1 & !bb2
                    } else if dr < 0 {
                        // s2 is below s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.south_attack(!bb2) & !bb1 & !bb2
                    }
                } else if same_rank {
                    if df > 0 {
                        // s2 is right of s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.east_attack(!bb2) & !bb1 & !bb2
                        // s2 is left of s1, equality already ruled out
                    } else if df < 0 {
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.west_attack(!bb2) & !bb1 & !bb2
                    }
                } else if df.abs() == dr.abs() {
                    // Diagonal
                    if df > 0 && dr > 0 {
                        // s2 is ne of s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.ne_attack(!bb2) & !bb1 & !bb2
                    } else if df < 0 && dr < 0 {
                        // s2 is sw of s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.sw_attack(!bb2) & !bb1 & !bb2
                    } else if df < 0 && dr > 0 {
                        // s2 is nw of s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.nw_attack(!bb2) & !bb1 & !bb2
                    } else if df > 0 && dr < 0 {
                        // s2 is se of s1
                        squares_between[s1.bitboard_index()][s2.bitboard_index()] =
                            bb1.se_attack(!bb2) & !bb1 & !bb2
                    }
                }
                // No cardinal or diagonal relationship, no squares between, next combination
            }
        }

        Self { squares_between }
    }

    pub fn between(&self, s1: Square, s2: Square) -> Bitboard {
        self.squares_between[s1.bitboard_index()][s2.bitboard_index()]
    }
}

impl Default for SquaresBetween {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn case_1() {
        let sb = SquaresBetween::new();

        let from = Square::new(File::F, Rank::R2);
        let to = Square::new(File::B, Rank::R2);

        let expected = Bitboard::from_squares(
            vec![
                Square::new(File::C, Rank::R2),
                Square::new(File::D, Rank::R2),
                Square::new(File::E, Rank::R2),
            ]
            .into_iter(),
        );

        assert_eq!(sb.between(from, to), expected)
    }
}
