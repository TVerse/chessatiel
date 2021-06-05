use crate::bitboard::Bitboard;
use crate::board::PieceArray;
use crate::color::Color;
use crate::file::File;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::Square;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PieceBoard {
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
    queens: Bitboard,
    kings: Bitboard,
}

impl Index<Piece> for PieceBoard {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawns,
            Piece::Knight => &self.knights,
            Piece::Bishop => &self.bishops,
            Piece::Rook => &self.rooks,
            Piece::Queen => &self.queens,
            Piece::King => &self.kings,
        }
    }
}

impl IndexMut<Piece> for PieceBoard {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        match index {
            Piece::Pawn => &mut self.pawns,
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook => &mut self.rooks,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
        }
    }
}

impl PieceBoard {
    pub const EMPTY: PieceBoard = PieceBoard {
        pawns: Bitboard(0),
        knights: Bitboard(0),
        bishops: Bitboard(0),
        rooks: Bitboard(0),
        queens: Bitboard(0),
        kings: Bitboard(0),
    };

    pub fn from_piecearray(pa: &PieceArray) -> (Self, Self) {
        let mut white = Self::EMPTY;
        let mut black = Self::EMPTY;

        for (r, file) in pa.0.iter().enumerate() {
            for (f, piece) in file.iter().enumerate() {
                if let Some((p, c)) = piece {
                    let bb = if *c == Color::White {
                        &mut white[*p]
                    } else {
                        &mut black[*p]
                    };

                    let square =
                        Square::new(File::from_u8_panic(f as u8), Rank::from_u8_panic(r as u8));

                    bb.set_mut(&square);
                }
            }
        }

        (white, black)
    }
}
