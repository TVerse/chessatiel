use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Not enough fields: got {0}, expected 6")]
    MissingField(usize),

    #[error("Wrong number of ranks: got {0}")]
    WrongNumberOfRanks(usize),
    #[error("Wrong number of files: got {0}")]
    WrongNumberOfFiles(usize),

    #[error("Invalid piece code: got {0}")]
    InvalidPiece(char),

    #[error("Invalid color: got {0}")]
    InvalidColor(String),

    #[error("Invalid integer")]
    InvalidInteger(#[from] ParseIntError),

    #[error("Invalid rank: got {0}")]
    InvalidRank(char),
    #[error("Invalid file: got {0}")]
    InvalidFile(char),
    #[error("Invalid square: got {0}")]
    InvalidSquare(String),
}
