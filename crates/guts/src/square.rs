use crate::bitboard::Bitboard;
use crate::file::File;
use crate::rank::Rank;
use crate::FenParseError;
#[cfg(test)]
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Square(u8);

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Square")
            .field("0", &self.0)
            .field("file", &self.file())
            .field("rank", &self.rank())
            .finish()
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Square {
    pub const NUM: usize = Self::ALL.len();

    pub const ALL: [Square; 64] = [
        Square(0),
        Square(1),
        Square(2),
        Square(3),
        Square(4),
        Square(5),
        Square(6),
        Square(7),
        Square(8),
        Square(9),
        Square(10),
        Square(11),
        Square(12),
        Square(13),
        Square(14),
        Square(15),
        Square(16),
        Square(17),
        Square(18),
        Square(19),
        Square(20),
        Square(21),
        Square(22),
        Square(23),
        Square(24),
        Square(25),
        Square(26),
        Square(27),
        Square(28),
        Square(29),
        Square(30),
        Square(31),
        Square(32),
        Square(33),
        Square(34),
        Square(35),
        Square(36),
        Square(37),
        Square(38),
        Square(39),
        Square(40),
        Square(41),
        Square(42),
        Square(43),
        Square(44),
        Square(45),
        Square(46),
        Square(47),
        Square(48),
        Square(49),
        Square(50),
        Square(51),
        Square(52),
        Square(53),
        Square(54),
        Square(55),
        Square(56),
        Square(57),
        Square(58),
        Square(59),
        Square(60),
        Square(61),
        Square(62),
        Square(63),
    ];

    pub fn new(file: File, rank: Rank) -> Self {
        let idx: u8 = (u8::from(rank) << 3) + u8::from(file);
        Self(idx)
    }

    pub fn from_index(idx: u8) -> Self {
        debug_assert!(idx < 64);
        Self(idx)
    }

    pub fn file(&self) -> File {
        File::from_u8_panic(self.0 & 0b111)
    }

    pub fn rank(&self) -> Rank {
        Rank::from_u8_panic(self.0 >> 3)
    }

    pub fn bitboard_index(&self) -> usize {
        self.0 as usize
    }

    pub fn ray_between(self, other: Square) -> Bitboard {
        /*
        Options:
        Equal: no result
        Same file or rank: cardinal(a) & cardinal(b)
        Different file and rank: diagonal(a) & diagonal(b)
         */
        let bb_self = Bitboard::from_square(self);
        let bb_other = Bitboard::from_square(other);
        if self.rank() == other.rank() && self.file() == other.file() {
            Bitboard::EMPTY
        } else if self.rank() == other.rank() || self.file() == other.file() {
            bb_self.cardinal_attackers(!bb_other) & bb_other.cardinal_attackers(!bb_self)
        } else {
            bb_self.diagonal_attackers(!bb_other) & bb_other.diagonal_attackers(!bb_self)
        }
    }
}

impl FromStr for Square {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err(FenParseError::InvalidSquare(s.to_owned()))
        } else {
            let mut chars = s.chars();
            let file = chars.next().unwrap();
            let rank = chars.next().unwrap();

            let file = File::try_from(file)?;
            let rank = Rank::try_from(rank)?;

            Ok(Self::new(file, rank))
        }
    }
}

#[cfg(test)]
impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl Ord for Square {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;

    #[test]
    fn pull_rank_file_back_out() {
        for rank in Rank::ALL.iter() {
            for file in File::ALL.iter() {
                let s = Square::new(*file, *rank);
                assert_eq!((s.rank(), s.file()), (*rank, *file));
            }
        }
    }

    #[test]
    fn complete_all() {
        for (idx, s) in Square::ALL.iter().enumerate() {
            assert_eq!(idx, s.0 as usize)
        }
    }
}
