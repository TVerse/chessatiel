#[macro_use]
extern crate bitflags;

pub use board::PieceBoard;
pub use chess_move::Move;
pub use color::Color;
pub use movegen::movebuffer::MoveBuffer;
pub use movegen::MoveGenerator;
pub use parse_error::FenParseError;
pub use piece::Piece;
pub use position::Position;

mod bitboard;
mod board;
mod castling_rights;
mod chess_move;
mod color;
pub mod fen;
mod file;
mod movegen;
mod parse_error;
mod piece;
mod position;
mod rank;
mod square;
