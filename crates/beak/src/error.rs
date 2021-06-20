use thiserror::Error;

#[derive(Error, Debug)]
pub enum UciParseError {
    #[error("Some error")]
    Error(String),
}
