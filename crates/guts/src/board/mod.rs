use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;
use crate::{Move, ParseError};
use std::convert::{TryFrom, TryInto};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

mod piece_board;

pub use piece_board::PieceBoard;

pub struct Sliders {
    pub cardinal: Bitboard,
    pub diagonal: Bitboard,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PieceArray(pub [[Option<(Piece, Color)>; 8]; 8]);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Board {
    white: PieceBoard,
    black: PieceBoard,
}

impl Index<Color> for Board {
    type Output = PieceBoard;

    fn index(&self, index: Color) -> &Self::Output {
        match index {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}

impl IndexMut<Color> for Board {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        match index {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
}

impl Board {
    pub fn from_piece_array(pieces: &PieceArray) -> Self {
        let (white, black) = PieceBoard::from_piecearray(pieces);
        Self { white, black }
    }

    fn update_piece(&self, pa: &mut PieceArray, color: Color, piece: Piece) {
        for s in self[color][piece].squares() {
            let rank: usize = u8::from(s.rank()) as usize;
            let file: usize = u8::from(s.file()) as usize;
            pa.0[7 - rank][file] = Some((piece, color))
        }
    }

    pub fn piece_array(&self) -> PieceArray {
        let mut pa = PieceArray([[None; 8]; 8]);
        for c in Color::ALL.iter() {
            for p in Piece::ALL.iter() {
                self.update_piece(&mut pa, *c, *p)
            }
        }

        pa
    }

    pub fn make_move(&mut self, chess_move: &Move, by: Color) {
        let piece = self.piece_at(chess_move.from).unwrap_or_else(|| {
            panic!("No piece found at move source square {}?", &chess_move.from)
        });
        for bb in self[by].bitboards.iter_mut() {
            *bb &= !Bitboard::from_square(chess_move.from);
        }
        self[by][piece] |= Bitboard::from_square(chess_move.to);
    }

    pub fn piece_at(&self, s: Square) -> Option<Piece> {
        self.white.piece_at(s).or_else(|| self.black.piece_at(s))
    }

    pub fn sliders(&self, color: Color) -> Sliders {
        self[color].sliders()
    }

    pub fn all_pieces(&self) -> Bitboard {
        self[Color::White].all_pieces() | self[Color::Black].all_pieces()
    }
}

impl From<Board> for PieceArray {
    fn from(b: Board) -> Self {
        b.piece_array()
    }
}

impl Default for Board {
    fn default() -> Self {
        use Color::*;
        use Piece::*;

        let pieces: PieceArray = PieceArray([
            [
                Some((Rook, Black)),
                Some((Knight, Black)),
                Some((Bishop, Black)),
                Some((Queen, Black)),
                Some((King, Black)),
                Some((Bishop, Black)),
                Some((Knight, Black)),
                Some((Rook, Black)),
            ],
            [Some((Pawn, Black)); 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [Some((Pawn, White)); 8],
            [
                Some((Rook, White)),
                Some((Knight, White)),
                Some((Bishop, White)),
                Some((Queen, White)),
                Some((King, White)),
                Some((Bishop, White)),
                Some((Knight, White)),
                Some((Rook, White)),
            ],
        ]);

        Self::from_piece_array(&pieces)
    }
}

impl FromStr for Board {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ranks = s.split('/');
        let ranks: Vec<_> = ranks.collect();
        let len = ranks.len();
        let ranks: [&str; 8] = ranks
            .try_into()
            .map_err(|_| ParseError::WrongNumberOfRanks(len))?;

        let mut pieces: PieceArray = PieceArray([[None; 8]; 8]);
        for (rank, target) in ranks.iter().zip(pieces.0.iter_mut()) {
            let chars = rank.chars();
            let mut ps: [Option<(Piece, Color)>; 8] = [None; 8];
            let mut idx: usize = 0;
            for c in chars {
                if let Some(n) = c.to_digit(10) {
                    idx += n as usize;
                } else {
                    let piece = Piece::try_from(c.to_ascii_uppercase())?;
                    let color = if c.is_ascii_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    ps[idx] = Some((piece, color));
                    idx += 1;
                }
            }
            if idx != 8 {
                return Err(ParseError::WrongNumberOfFiles(idx));
            }
            *target = ps;
        }

        // TODO skip the PieceArray
        Ok(Board::from_piece_array(&pieces))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_array_transform_is_reversible() {
        use Color::*;
        use Piece::*;

        let expected = PieceArray([
            [
                Some((Rook, Black)),
                Some((Knight, Black)),
                Some((Bishop, Black)),
                Some((Queen, Black)),
                Some((King, Black)),
                Some((Bishop, Black)),
                Some((Knight, Black)),
                Some((Rook, Black)),
            ],
            [Some((Pawn, Black)); 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [Some((Pawn, White)); 8],
            [
                Some((Rook, White)),
                Some((Knight, White)),
                Some((Bishop, White)),
                Some((Queen, White)),
                Some((King, White)),
                Some((Bishop, White)),
                Some((Knight, White)),
                Some((Rook, White)),
            ],
        ]);

        assert_eq!(Board::from_piece_array(&expected).piece_array(), expected)
    }
}
