use crate::square::Square;

#[derive(Debug)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn is_set(&self, i: Index) -> bool {
        let mask = 1 << i.0;
        self.0 & mask != 0
    }
}

#[derive(Debug)]
pub struct Index(u8);

impl From<&Square> for Index {
    fn from(s: &Square) -> Self {
        let idx: u8 = 8u8 * u8::from(s.rank()) + u8::from(s.file());
        Self(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn check_some_squares() {
        let board = Bitboard(0x00_00_A0_00_00_00_01_01);
        let set = [
            Square::new(File::A, Rank::R1),
            Square::new(File::A, Rank::R2),
            Square::new(File::F, Rank::R6),
            Square::new(File::H, Rank::R6),
        ];
        for file in File::ALL.iter() {
            for rank in Rank::ALL.iter() {
                let square = Square::new(*file, *rank);
                if set.contains(&square) {
                    assert!(board.is_set((&square).into()), "square = {:?}", &square);
                } else {
                    assert!(!board.is_set((&square).into()), "square = {:?}", square);
                }
            }
        }
    }
}
