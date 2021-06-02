use crate::file::File;
use crate::rank::Rank;
use crate::ParseError;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Square {
    file: File,
    rank: Rank,
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    pub fn file(&self) -> File {
        self.file
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

impl FromStr for Square {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err(ParseError::InvalidSquare(s.to_owned()))
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
