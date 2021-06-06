use crate::file::File;
use crate::rank::Rank;
use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Bitboard(pub u64);

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0.to_be_bytes();
        writeln!(f)?;
        for b in bytes.iter() {
            writeln!(f, "{:08b}", b.reverse_bits())?
        }
        Ok(())
    }
}

impl Bitboard {
    pub fn is_set(&self, s: &Square) -> bool {
        let mask = 1 << s.bitboard_index();
        self.0 & mask != 0
    }

    pub fn set_mut(&mut self, s: &Square) {
        let mask = 1 << s.bitboard_index();
        self.0 |= mask
    }

    pub fn squares(&'_ self) -> impl Iterator<Item = Square> + '_ {
        Rank::ALL.iter().flat_map(move |r| {
            File::ALL.iter().filter_map(move |f| {
                let s = Square::new(*f, *r);
                if self.is_set(&s) {
                    Some(s)
                } else {
                    None
                }
            })
        })
    }

    pub fn from_squares_ref<'a, I: Iterator<Item = &'a Square>>(squares: I) -> Self {
        let mut bb = Bitboard(0);
        for s in squares {
            bb.set_mut(s);
        }
        bb
    }

    pub fn from_squares<I: Iterator<Item = Square>>(squares: I) -> Self {
        let mut bb = Bitboard(0);
        for s in squares {
            bb.set_mut(&s);
        }
        bb
    }

    pub fn from_square(square: &Square) -> Self {
        Self::from_squares_ref(std::iter::once(square))
    }

    pub fn first_set_square(&self) -> Square {
        let idx = self.0.trailing_zeros() as u8;
        Square::from_index(idx)
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

pub struct BitboardIterator {
    bitboard: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard.0 == 0 {
            None
        } else {
            let s = self.bitboard.first_set_square();
            self.bitboard ^= Bitboard::from_square(&s);
            Some(s)
        }
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIterator { bitboard: self }
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

    #[test]
    fn iterator() {
        let board = Bitboard(0xFF);
        let squares: Vec<_> = board.into_iter().collect();
        assert_eq!(squares.len(), 8);
        let ranks = squares.iter().map(|s| s.rank());
        for r in ranks {
            assert_eq!(r, Rank::R1)
        }
        let files: Vec<_> = squares.iter().map(|s| s.file()).collect();
        for f in File::ALL.iter() {
            assert!(files.contains(f))
        }

        let board = Bitboard(0xFF00);
        let squares: Vec<_> = board.into_iter().collect();
        assert_eq!(squares.len(), 8);
        let ranks = squares.iter().map(|s| s.rank());
        for r in ranks {
            assert_eq!(r, Rank::R2)
        }
        let files: Vec<_> = squares.iter().map(|s| s.file()).collect();
        for f in File::ALL.iter() {
            assert!(files.contains(f))
        }
    }

    #[test]
    fn iterator_all_squares() {
        let board = Bitboard(u64::MAX);
        assert_eq!(board.into_iter().collect::<Vec<_>>().len(), 64)
    }

    macro_rules! test_operator {
        ($fn_name:literal) => {
            paste::item! {
                #[test]
                fn [< test_ $fn_name >] () {
                    let bb1 = Bitboard(0x0123456787654321);
                    let bb2 = Bitboard(0xFEDCBA9087654321);

                    assert_eq!((bb1.[< $fn_name >](bb2)).0, bb1.0.[< $fn_name >](bb2.0));
                }

                #[test]
                fn [< test_ $fn_name _assign >] () {
                    let mut bb1 = Bitboard(0x0123456787654321);
                    let bb2 = Bitboard(0xFEDCBA9087654321);

                    let mut u1 = 0x0123456787654321;
                    let u2 = 0xFEDCBA9087654321;

                    bb1.[< $fn_name _assign>](bb2);
                    u1.[< $fn_name _assign>](u2);

                    assert_eq!(bb1.0, u1);
                }
            }
        };
    }

    test_operator!("bitxor");
    test_operator!("bitor");
    test_operator!("bitand");

    #[test]
    fn test_not() {
        let bb = Bitboard(0x0123456787654321);

        assert_eq!((!bb).0, !(bb.0));
    }
}
