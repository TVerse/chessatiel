use crate::bitboard::Bitboard;
use crate::chess_move::MoveType;
use crate::square::Square;
use crate::{Move, Piece};
use std::iter::FusedIterator;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Priority {
    Captures,
    Pushes,
    Castles,
}

#[derive(Debug, Copy, Clone)]
pub struct Priorities {
    inner: [Priority; 3],
}

impl Priorities {
    pub fn new(first: Priority, second: Priority, third: Priority) -> Self {
        Self {
            inner: [first, second, third],
        }
    }
}

impl Default for Priorities {
    fn default() -> Self {
        Self::new(Priority::Captures, Priority::Castles, Priority::Pushes)
    }
}

#[derive(Debug)]
pub struct MoveBuffer {
    captures: Vec<Move>,
    pushes: Vec<Move>,
    castles: Vec<Move>,
}

impl MoveBuffer {
    pub fn new() -> Self {
        Self {
            captures: Vec::with_capacity(50),
            pushes: Vec::with_capacity(50),
            castles: Vec::with_capacity(2),
        }
    }

    pub fn add_push(&mut self, piece: Piece, from: Square, targets: Bitboard) {
        self.pushes.extend(
            targets
                .into_iter()
                .map(|to| Move::new(from, to, piece, MoveType::PUSH, None)),
        )
    }

    pub fn add_pawn_push(&mut self, from: Square, targets: Bitboard) {
        let promotion_pawns = targets & (Bitboard::RANK_1 | Bitboard::RANK_8);
        let not_promotion_pawns = targets & !promotion_pawns;

        self.pushes.extend(
            not_promotion_pawns
                .into_iter()
                .map(|to| Move::new(from, to, Piece::Pawn, MoveType::PUSH, None)),
        );

        self.pushes
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

        self.captures.extend(
            not_promotion_pawns
                .into_iter()
                .map(|to| Move::new(from, to, Piece::Pawn, MoveType::CAPTURE, None)),
        );

        self.captures
            .extend(promotion_pawns.into_iter().flat_map(|to| {
                Piece::PROMOTION_TARGETS
                    .iter()
                    .copied()
                    .map(move |pt| Move::new(from, to, Piece::Pawn, MoveType::CAPTURE, Some(pt)))
            }));
    }

    pub fn add_capture(&mut self, piece: Piece, from: Square, targets: Bitboard) {
        self.captures.extend(
            targets
                .into_iter()
                .map(|to| Move::new(from, to, piece, MoveType::CAPTURE, None)),
        )
    }

    pub fn add_en_passant(&mut self, from: Square, targets: Bitboard) {
        self.captures.extend(targets.into_iter().map(|to| {
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
        self.castles
            .push(Move::new(from, to, Piece::King, move_type, None))
    }

    pub fn len(&self) -> usize {
        self.castles.len() + self.pushes.len() + self.captures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.castles.is_empty() && self.pushes.is_empty() && self.captures.is_empty()
    }

    pub fn clear(&mut self) {
        self.captures.clear();
        self.castles.clear();
        self.pushes.clear();
    }

    pub fn iter(&self) -> MoveIterator<'_> {
        self.priority_iter(Priorities::default())
    }

    pub fn priority_iter(&self, priorities: Priorities) -> MoveIterator<'_> {
        MoveIterator::new(self, priorities)
    }

    fn for_priority(&self, priority: Priority) -> &[Move] {
        match priority {
            Priority::Captures => &self.captures,
            Priority::Pushes => &self.pushes,
            Priority::Castles => &self.castles,
        }
    }
}

impl Default for MoveBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MoveIterator<'a> {
    buf: &'a MoveBuffer,
    priorities: Priorities,
    cur_priority: usize,
    idx: usize,
    len: usize,
}

impl<'a> MoveIterator<'a> {
    pub fn new(buf: &'a MoveBuffer, priorities: Priorities) -> Self {
        let len = buf.for_priority(priorities.inner[0]).len();
        Self {
            buf,
            priorities,
            cur_priority: 0,
            idx: 0,
            len,
        }
    }
}

impl<'a> Iterator for MoveIterator<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_priority >= self.priorities.inner.len() {
            None
        } else if self.idx >= self.len {
            self.cur_priority += 1;
            if self.cur_priority >= self.priorities.inner.len() {
                None
            } else {
                self.idx = 0;
                self.len = self
                    .buf
                    .for_priority(self.priorities.inner[self.cur_priority])
                    .len();
                self.next()
            }
        } else {
            self.idx += 1;
            Some(
                &self
                    .buf
                    .for_priority(self.priorities.inner[self.cur_priority])[self.idx - 1],
            )
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for MoveIterator<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a> FusedIterator for MoveIterator<'a> {}
