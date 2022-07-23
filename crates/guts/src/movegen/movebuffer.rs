use crate::bitboard::Bitboard;
use crate::chess_move::MoveType;
use crate::square::Square;
use crate::{Move, Piece};

#[derive(Debug)]
pub struct MoveBuffer {
    moves: Vec<Move>,
}

impl MoveBuffer {
    pub fn new() -> Self {
        Self {
            moves: Vec::with_capacity(50),
        }
    }

    pub fn add_push(&mut self, piece: Piece, from: Square, targets: Bitboard) {
        self.moves.extend(
            targets
                .into_iter()
                .map(|to| Move::new(from, to, piece, MoveType::PUSH, None)),
        )
    }

    pub fn add_pawn_push(&mut self, from: Square, targets: Bitboard) {
        let promotion_pawns = targets & (Bitboard::RANK_1 | Bitboard::RANK_8);
        let not_promotion_pawns = targets & !promotion_pawns;

        self.moves.extend(
            not_promotion_pawns
                .into_iter()
                .map(|to| Move::new(from, to, Piece::Pawn, MoveType::PUSH, None)),
        );

        self.moves
            .extend(promotion_pawns.into_iter().flat_map(|to| {
                Piece::PROMOTION_TARGETS
                    .iter()
                    .copied()
                    .map(move |pt| Move::new(from, to, Piece::Pawn, MoveType::PUSH, Some(pt)))
            }));
    }

    pub fn add_pawn_capture(&mut self, from: Square, targets: Bitboard) {
        let promotion_pawns = targets & (Bitboard::RANK_1 | Bitboard::RANK_8);
        let not_promotion_pawns = targets & !promotion_pawns;

        self.moves.extend(
            not_promotion_pawns
                .into_iter()
                .map(|to| Move::new(from, to, Piece::Pawn, MoveType::CAPTURE, None)),
        );

        self.moves
            .extend(promotion_pawns.into_iter().flat_map(|to| {
                Piece::PROMOTION_TARGETS
                    .iter()
                    .copied()
                    .map(move |pt| Move::new(from, to, Piece::Pawn, MoveType::CAPTURE, Some(pt)))
            }));
    }

    pub fn add_capture(&mut self, piece: Piece, from: Square, targets: Bitboard) {
        self.moves.extend(
            targets
                .into_iter()
                .map(|to| Move::new(from, to, piece, MoveType::CAPTURE, None)),
        )
    }

    pub fn add_en_passant(&mut self, from: Square, targets: Bitboard) {
        self.moves.extend(targets.into_iter().map(|to| {
            Move::new(
                from,
                to,
                Piece::Pawn,
                MoveType::CAPTURE | MoveType::EN_PASSANT,
                None,
            )
        }))
    }

    pub fn add_castle(&mut self, from: Square, to: Square, move_type: MoveType) {
        self.moves
            .push(Move::new(from, to, Piece::King, move_type, None))
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn clear(&mut self) {
        self.moves.clear()
    }

    pub fn iter(&self) -> MoveIterator<'_> {
        MoveIterator::new(self)
    }
}

impl Default for MoveBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MoveIterator<'a> {
    buf: &'a MoveBuffer,
    idx: usize,
    len: usize,
}

impl<'a> MoveIterator<'a> {
    pub fn new(buf: &'a MoveBuffer) -> Self {
        Self {
            buf,
            idx: 0,
            len: buf.len(),
        }
    }
}

impl<'a> Iterator for MoveIterator<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            self.idx += 1;
            Some(&self.buf.moves[self.idx - 1])
        }
    }
}
