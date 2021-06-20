use crate::FenParseError;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
        };

        write!(f, "{}", c)
    }
}

impl File {
    pub const ALL: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    pub const INNER: [File; 6] = [File::B, File::C, File::D, File::E, File::F, File::G];

    pub fn from_u8_panic(i: u8) -> Self {
        match i {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => panic!("No file for index {}", i),
        }
    }

    pub fn index(&self) -> usize {
        u8::from(*self) as usize
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            File::A => None,
            File::B => Some(File::A),
            File::C => Some(File::B),
            File::D => Some(File::C),
            File::E => Some(File::D),
            File::F => Some(File::E),
            File::G => Some(File::F),
            File::H => Some(File::G),
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            File::A => Some(File::B),
            File::B => Some(File::C),
            File::C => Some(File::D),
            File::D => Some(File::E),
            File::E => Some(File::F),
            File::F => Some(File::G),
            File::G => Some(File::H),
            File::H => None,
        }
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        u8::from(*self).cmp(&u8::from(*other))
    }
}

impl From<File> for u8 {
    fn from(f: File) -> Self {
        match f {
            File::A => 0,
            File::B => 1,
            File::C => 2,
            File::D => 3,
            File::E => 4,
            File::F => 5,
            File::G => 6,
            File::H => 7,
        }
    }
}

impl TryFrom<char> for File {
    type Error = FenParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a' => Ok(File::A),
            'b' => Ok(File::B),
            'c' => Ok(File::C),
            'd' => Ok(File::D),
            'e' => Ok(File::E),
            'f' => Ok(File::F),
            'g' => Ok(File::G),
            'h' => Ok(File::H),
            _ => Err(FenParseError::InvalidFile(c)),
        }
    }
}
