use crate::bitboard::Bitboard;
use crate::board::{PieceArray, Sliders};
use crate::color::Color;
use crate::file::File;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::Square;
use std::ops::{Index, IndexMut};

// TODO try to implement iter_mut instead of public field
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PieceBoard {
    pub bitboards: [Bitboard; 6],
}

impl Index<Piece> for PieceBoard {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self.bitboards[index.index()]
    }
}

impl IndexMut<Piece> for PieceBoard {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.bitboards[index.index()]
    }
}

impl PieceBoard {
    pub const EMPTY: PieceBoard = PieceBoard {
        bitboards: [Bitboard::EMPTY; 6],
    };

    pub fn from_piecearray(pa: &PieceArray) -> (Self, Self) {
        let mut white = Self::EMPTY;
        let mut black = Self::EMPTY;

        for (r, file) in pa.0.iter().rev().enumerate() {
            for (f, piece) in file.iter().enumerate() {
                if let Some((p, c)) = piece {
                    let bb = if *c == Color::White {
                        &mut white[*p]
                    } else {
                        &mut black[*p]
                    };

                    let square =
                        Square::new(File::from_u8_panic(f as u8), Rank::from_u8_panic(r as u8));

                    bb.set_mut(square);
                }
            }
        }

        (white, black)
    }

    pub fn all_pieces(&self) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        for p in Piece::ALL.iter() {
            bb |= self[*p];
        }
        bb
    }

    pub fn sliders(&self) -> Sliders {
        let cardinal = self[Piece::Rook] | self[Piece::Queen];
        let diagonal = self[Piece::Bishop] | self[Piece::Queen];

        Sliders { cardinal, diagonal }
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for (idx, bb) in self.bitboards.iter().enumerate() {
            if bb.is_set(square) {
                return Some(Piece::from_usize_panic(idx));
            }
        }
        None
    }
}
