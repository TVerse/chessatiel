use thiserror::Error;

#[derive(Error, Debug)]
pub enum UciParseError {
    #[error("Error: {0}")]
    Error(String),
}
