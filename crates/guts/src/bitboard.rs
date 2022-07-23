use crate::color::Color;
use crate::file::File;
use crate::rank::Rank;
use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

// Occluded by Krogge-Stone algorithm
// https://www.chessprogramming.org/Kogge-Stone_Algorithm

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Bitboard(u64);

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        let bytes = self.0.to_be_bytes();
        for b in bytes.iter() {
            writeln!(f, "{:08b}", b.reverse_bits())?
        }
        Ok(())
    }
}

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(u64::MAX);

    pub const A_FILE: Bitboard = Bitboard(0x01_01_01_01_01_01_01_01);
    pub const B_FILE: Bitboard = Bitboard(0x02_02_02_02_02_02_02_02);
    pub const C_FILE: Bitboard = Bitboard(0x04_04_04_04_04_04_04_04);
    pub const D_FILE: Bitboard = Bitboard(0x08_08_08_08_08_08_08_08);
    pub const E_FILE: Bitboard = Bitboard(0x10_10_10_10_10_10_10_10);
    pub const F_FILE: Bitboard = Bitboard(0x20_20_20_20_20_20_20_20);
    pub const G_FILE: Bitboard = Bitboard(0x40_40_40_40_40_40_40_40);
    pub const H_FILE: Bitboard = Bitboard(0x80_80_80_80_80_80_80_80);
    pub const RANK_1: Bitboard = Bitboard(0x00_00_00_00_00_00_00_FF);
    pub const RANK_2: Bitboard = Bitboard(0x00_00_00_00_00_00_FF_00);
    pub const RANK_3: Bitboard = Bitboard(0x00_00_00_00_00_FF_00_00);
    pub const RANK_4: Bitboard = Bitboard(0x00_00_00_00_FF_00_00_00);
    pub const RANK_5: Bitboard = Bitboard(0x00_00_00_FF_00_00_00_00);
    pub const RANK_6: Bitboard = Bitboard(0x00_00_FF_00_00_00_00_00);
    pub const RANK_7: Bitboard = Bitboard(0x00_FF_00_00_00_00_00_00);
    pub const RANK_8: Bitboard = Bitboard(0xFF_00_00_00_00_00_00_00);

    pub fn is_set(self, s: Square) -> bool {
        let mask = 1 << s.bitboard_index();
        self.0 & mask != 0
    }

    pub fn set_mut(&mut self, s: Square) {
        let mask = 1 << s.bitboard_index();
        self.0 |= mask
    }

    pub fn clear_mut(&mut self, s: Square) {
        let mask = 1 << s.bitboard_index();
        self.0 &= !mask
    }

    pub fn squares(self) -> impl Iterator<Item = Square> {
        Rank::ALL.iter().flat_map(move |r| {
            File::ALL.iter().filter_map(move |f| {
                let s = Square::new(*f, *r);
                if self.is_set(s) {
                    Some(s)
                } else {
                    None
                }
            })
        })
    }

    pub fn from_squares<I: Iterator<Item = Square>>(squares: I) -> Self {
        let mut bb = Bitboard::EMPTY;
        for s in squares {
            bb.set_mut(s);
        }
        bb
    }

    // TODO impl From<Square> for Bitboard
    pub fn from_square(square: Square) -> Self {
        Self::from_squares(std::iter::once(square))
    }

    pub fn first_set_square(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let idx = self.0.trailing_zeros() as u8;
            Some(Square::from_index(idx))
        }
    }

    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    pub fn cardinal_attackers(self, empty: Self) -> Self {
        self.north_attack(empty)
            | self.east_attack(empty)
            | self.south_attack(empty)
            | self.west_attack(empty)
    }

    pub fn west_attack(self, empty: Bitboard) -> Bitboard {
        self.west_occluded(empty).west_one()
    }

    pub fn south_attack(self, empty: Bitboard) -> Bitboard {
        self.south_occluded(empty).south_one()
    }

    pub fn east_attack(self, empty: Bitboard) -> Bitboard {
        self.east_occluded(empty).east_one()
    }

    pub fn north_attack(self, empty: Bitboard) -> Bitboard {
        self.north_occluded(empty).north_one()
    }

    pub fn diagonal_attackers(self, empty: Self) -> Self {
        self.ne_attack(empty)
            | self.nw_attack(empty)
            | self.se_attack(empty)
            | self.sw_attack(empty)
    }

    pub fn sw_attack(self, empty: Bitboard) -> Bitboard {
        self.sw_occluded(empty).sw_one()
    }

    pub fn se_attack(self, empty: Bitboard) -> Bitboard {
        self.se_occluded(empty).se_one()
    }

    pub fn nw_attack(self, empty: Bitboard) -> Bitboard {
        self.nw_occluded(empty).nw_one()
    }

    pub fn ne_attack(self, empty: Bitboard) -> Bitboard {
        self.ne_occluded(empty).ne_one()
    }

    pub fn south_one(self) -> Self {
        Self(self.0 >> 8)
    }

    pub fn north_one(self) -> Self {
        Self(self.0 << 8)
    }

    pub fn east_one(self) -> Self {
        Self(self.0 << 1) & !Self::A_FILE
    }

    pub fn west_one(self) -> Self {
        Self(self.0 >> 1) & !Self::H_FILE
    }

    pub fn se_one(self) -> Self {
        Self(self.0 >> 7) & !Self::A_FILE
    }

    pub fn sw_one(self) -> Self {
        Self(self.0 >> 9) & !Self::H_FILE
    }

    pub fn ne_one(self) -> Self {
        Self(self.0 << 9) & !Self::A_FILE
    }

    pub fn nw_one(self) -> Self {
        Self(self.0 << 7) & !Self::H_FILE
    }

    pub fn south_occluded(self, empty: Self) -> Self {
        let mut gen = self.0;
        let mut pro = empty.0;

        gen |= pro & (gen >> 8);
        pro &= pro >> 8;
        gen |= pro & (gen >> 16);
        pro &= pro >> 16;
        gen |= pro & (gen >> 32);

        Self(gen)
    }

    pub fn north_occluded(self, empty: Self) -> Self {
        let mut gen = self.0;
        let mut pro = empty.0;

        gen |= pro & (gen << 8);
        pro &= pro << 8;
        gen |= pro & (gen << 16);
        pro &= pro << 16;
        gen |= pro & (gen << 32);

        Self(gen)
    }

    pub fn east_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::A_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen << 1);
        pro &= pro << 1;
        gen |= pro & (gen << 2);
        pro &= pro << 2;
        gen |= pro & (gen << 4);

        Self(gen)
    }

    pub fn west_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::H_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen >> 1);
        pro &= pro >> 1;
        gen |= pro & (gen >> 2);
        pro &= pro >> 2;
        gen |= pro & (gen >> 4);

        Self(gen)
    }

    pub fn se_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::A_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen >> 7);
        pro &= pro >> 7;
        gen |= pro & (gen >> 14);
        pro &= pro >> 14;
        gen |= pro & (gen >> 28);

        Self(gen)
    }
    pub fn sw_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::H_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen >> 9);
        pro &= pro >> 9;
        gen |= pro & (gen >> 18);
        pro &= pro >> 18;
        gen |= pro & (gen >> 36);

        Self(gen)
    }
    pub fn ne_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::A_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen << 9);
        pro &= pro << 9;
        gen |= pro & (gen << 18);
        pro &= pro << 18;
        gen |= pro & (gen << 36);

        Self(gen)
    }
    pub fn nw_occluded(self, empty: Self) -> Self {
        let mut pro = (empty & !Self::H_FILE).0;
        let mut gen = self.0;

        gen |= pro & (gen << 7);
        pro &= pro << 7;
        gen |= pro & (gen << 14);
        pro &= pro << 14;
        gen |= pro & (gen << 28);

        Self(gen)
    }

    pub fn forward_one(self, color: Color) -> Self {
        match color {
            Color::White => self.north_one(),
            Color::Black => self.south_one(),
        }
    }

    pub fn forward_left_one(self, color: Color) -> Self {
        match color {
            Color::White => self.nw_one(),
            Color::Black => self.se_one(),
        }
    }

    pub fn forward_right_one(self, color: Color) -> Self {
        match color {
            Color::White => self.ne_one(),
            Color::Black => self.sw_one(),
        }
    }

    pub fn surrounding(self) -> Self {
        self.north_one()
            | self.ne_one()
            | self.east_one()
            | self.se_one()
            | self.south_one()
            | self.sw_one()
            | self.west_one()
            | self.nw_one()
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
        self.bitboard.first_set_square().map(|s| {
            self.bitboard ^= Bitboard::from_square(s);
            s
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.bitboard.count_ones() as usize,
            Some(self.bitboard.count_ones() as usize),
        )
    }
}

impl ExactSizeIterator for BitboardIterator {
    fn len(&self) -> usize {
        self.bitboard.count_ones() as usize
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
                    assert!(board.is_set(square), "square = {:?}", &square);
                } else {
                    assert!(!board.is_set(square), "square = {:?}", square);
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
        let mut files = squares.iter().map(|s| s.file());
        for f in File::ALL.iter() {
            assert!(files.any(|x| x == *f));
        }

        let board = Bitboard(0xFF00);
        let squares: Vec<_> = board.into_iter().collect();
        assert_eq!(squares.len(), 8);
        let ranks = squares.iter().map(|s| s.rank());
        for r in ranks {
            assert_eq!(r, Rank::R2)
        }
        let mut files = squares.iter().map(|s| s.file());
        for f in File::ALL.iter() {
            assert!(files.any(|x| x == *f))
        }
    }

    #[test]
    fn krogge_stone_cardinal() {
        let rooks = Bitboard::from_squares(
            vec![
                Square::new(File::E, Rank::R5),
                Square::new(File::C, Rank::R8),
            ]
            .into_iter(),
        );

        let blockers = Bitboard::from_squares(
            vec![
                Square::new(File::B, Rank::R5),
                Square::new(File::G, Rank::R5),
                Square::new(File::E, Rank::R8),
                Square::new(File::E, Rank::R3),
                Square::new(File::C, Rank::R7),
            ]
            .into_iter(),
        );
        let empty = !blockers;

        let expected_result = Bitboard::from_squares(
            vec![
                Square::new(File::B, Rank::R5),
                Square::new(File::C, Rank::R5),
                Square::new(File::D, Rank::R5),
                Square::new(File::F, Rank::R5),
                Square::new(File::G, Rank::R5),
                Square::new(File::E, Rank::R3),
                Square::new(File::E, Rank::R4),
                Square::new(File::E, Rank::R6),
                Square::new(File::E, Rank::R7),
                Square::new(File::E, Rank::R8),
                Square::new(File::A, Rank::R8),
                Square::new(File::B, Rank::R8),
                Square::new(File::C, Rank::R7),
                Square::new(File::D, Rank::R8),
            ]
            .into_iter(),
        );

        assert_eq!(rooks.cardinal_attackers(empty), expected_result);
    }

    #[test]
    fn krogge_stone_diagonal() {
        let bishops = Bitboard::from_square(Square::new(File::E, Rank::R5));

        let blockers = Bitboard::from_squares(
            vec![
                Square::new(File::C, Rank::R7),
                Square::new(File::F, Rank::R6),
                Square::new(File::E, Rank::R2),
                Square::new(File::H, Rank::R2),
            ]
            .into_iter(),
        );
        let empty = !blockers;

        let expected_result = Bitboard::from_squares(
            vec![
                Square::new(File::C, Rank::R7),
                Square::new(File::D, Rank::R6),
                Square::new(File::F, Rank::R6),
                Square::new(File::H, Rank::R2),
                Square::new(File::G, Rank::R3),
                Square::new(File::F, Rank::R4),
                Square::new(File::A, Rank::R1),
                Square::new(File::B, Rank::R2),
                Square::new(File::C, Rank::R3),
                Square::new(File::D, Rank::R4),
            ]
            .into_iter(),
        );

        assert_eq!(bishops.diagonal_attackers(empty), expected_result)
    }

    #[test]
    fn krogge_stone_diagonal_bug_1() {
        let from =
            Bitboard(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100);
        let empty =
            Bitboard(0b00000000_00001000_11111111_11110111_11111111_11111101_00000010_00000000);

        let expected = Bitboard::from_squares(
            vec![
                Square::new(File::C, Rank::R1),
                Square::new(File::B, Rank::R2),
                Square::new(File::A, Rank::R3),
            ]
            .into_iter(),
        );

        assert_eq!(from.nw_occluded(empty), expected)
    }
    #[test]
    fn iterator_all_squares() {
        let board = Bitboard(u64::MAX);
        assert_eq!(board.into_iter().count(), 64)
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
