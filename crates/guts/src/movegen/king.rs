use crate::chess_move::ExtraMoveInfo;
use crate::movegen::Masks;
use crate::{Move, Piece, Position};

pub(super) fn move_for_king(position: &Position, masks: &Masks) -> Vec<Move> {
    let own_pieces = &position.board()[position.active_color()];
    let king = own_pieces[Piece::King];
    let king_square = king.first_set_square().unwrap();

    let candidate_squares = king.surrounding();

    let possible_squares = (candidate_squares & !masks.king_danger) & !own_pieces.all_pieces();
    possible_squares
        .into_iter()
        .map(|s| Move::new(king_square, s, Some(ExtraMoveInfo::new(Piece::King))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::bitboard::Bitboard;

    #[test]
    fn king_in_corner_no_masks() {
        let position = Position::from_str("8/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let masks = Masks::EMPTY;

        let result = move_for_king(&position, &masks);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn king_in_corner_cut_off() {
        let position = Position::from_str("1r6/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let masks = Masks::new(Bitboard::B_FILE, Bitboard::EMPTY, Bitboard::EMPTY);

        let result = move_for_king(&position, &masks);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn king_boxed_in() {
        let position = Position::from_str("1r6/8/8/8/8/8/PN6/KN6 w - - 0 1").unwrap();
        let masks = Masks::new(Bitboard::B_FILE, Bitboard::EMPTY, Bitboard::EMPTY);

        let result = move_for_king(&position, &masks);

        assert_eq!(result.len(), 0);
    }
}
