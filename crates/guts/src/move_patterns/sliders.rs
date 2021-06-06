use crate::bitboard::Bitboard;
use crate::square::Square;

// TODO magic bitboards

enum CardinalDirection {
    Up,
    Down,
    Left,
    Right,
}

fn search_direction_sequence(
    source: &Square,
    occupancy: &Bitboard,
    sequence: &[CardinalDirection],
) -> Bitboard {
    let mut bb = Bitboard(0);

    let mut cur = Some(*source);

    let squares = std::iter::from_fn(|| {
        let next = sequence.iter().fold(cur, |acc, cd| match cd {
            CardinalDirection::Up => {
                acc.and_then(|acc| acc.rank().next().map(|rank| Square::new(acc.file(), rank)))
            }
            CardinalDirection::Down => {
                acc.and_then(|acc| acc.rank().prev().map(|rank| Square::new(acc.file(), rank)))
            }
            CardinalDirection::Left => {
                acc.and_then(|acc| acc.file().prev().map(|file| Square::new(file, acc.rank())))
            }
            CardinalDirection::Right => {
                acc.and_then(|acc| acc.file().next().map(|file| Square::new(file, acc.rank())))
            }
        });

        cur = next;

        next
    });

    for s in squares {
        bb.set_mut(&s);
        if occupancy.is_set(&s) {
            break;
        }
    }

    bb
}

fn get_rook(source: &Square, occupancy: &Bitboard) -> Bitboard {
    search_direction_sequence(source, occupancy, vec![CardinalDirection::Up].as_slice())
        | search_direction_sequence(source, occupancy, vec![CardinalDirection::Left].as_slice())
        | search_direction_sequence(source, occupancy, vec![CardinalDirection::Down].as_slice())
        | search_direction_sequence(source, occupancy, vec![CardinalDirection::Right].as_slice())
}

fn get_bishop(source: &Square, occupancy: &Bitboard) -> Bitboard {
    search_direction_sequence(
        source,
        occupancy,
        vec![CardinalDirection::Up, CardinalDirection::Left].as_slice(),
    ) | search_direction_sequence(
        source,
        occupancy,
        vec![CardinalDirection::Left, CardinalDirection::Down].as_slice(),
    ) | search_direction_sequence(
        source,
        occupancy,
        vec![CardinalDirection::Down, CardinalDirection::Right].as_slice(),
    ) | search_direction_sequence(
        source,
        occupancy,
        vec![CardinalDirection::Right, CardinalDirection::Up].as_slice(),
    )
}

fn get_queen(source: &Square, occupancy: &Bitboard) -> Bitboard {
    get_rook(source, occupancy) | get_bishop(source, occupancy)
}

#[derive(Default)]
pub struct BishopMovePatterns {}

impl BishopMovePatterns {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_move(&self, s: &Square, occupancy: &Bitboard) -> Bitboard {
        get_bishop(s, occupancy)
    }
}

#[derive(Default)]
pub struct RookMovePatterns {}

impl RookMovePatterns {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_move(&self, s: &Square, occupancy: &Bitboard) -> Bitboard {
        get_rook(s, occupancy)
    }
}

#[derive(Default)]
pub struct QueenMovePatterns {}

impl QueenMovePatterns {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_move(&self, s: &Square, occupancy: &Bitboard) -> Bitboard {
        get_queen(s, occupancy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::File;
    use crate::rank::Rank;
    use crate::square::Square;

    #[test]
    fn check_some_squares_rook() {
        let m = RookMovePatterns::new();

        let starting_square = Square::new(File::A, Rank::R1);
        let expected_squares = vec![
            Square::new(File::A, Rank::R2),
            Square::new(File::A, Rank::R3),
            Square::new(File::A, Rank::R4),
            Square::new(File::A, Rank::R5),
            Square::new(File::A, Rank::R6),
            Square::new(File::A, Rank::R7),
            Square::new(File::A, Rank::R8),
            Square::new(File::B, Rank::R1),
            Square::new(File::C, Rank::R1),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square, &Bitboard(4));

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_move_count_rook() {
        let m = RookMovePatterns::new();
        for s in Square::ALL.iter() {
            let res = m.get_move(s, &Bitboard(0)); // should not panic
            assert_eq!(res.count_ones(), 14)
        }
    }

    #[test]
    fn check_some_squares_bishop() {
        let m = BishopMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let expected_squares = vec![
            Square::new(File::A, Rank::R1),
            Square::new(File::C, Rank::R3),
            Square::new(File::D, Rank::R4),
            Square::new(File::C, Rank::R1),
            Square::new(File::A, Rank::R3),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(
            &starting_square,
            &Bitboard::from_square(&Square::new(File::D, Rank::R4)),
        );

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics_bishop() {
        let m = BishopMovePatterns::new();
        for s in Square::ALL.iter() {
            let _res = m.get_move(s, &Bitboard(0)); // should not panic
        }
    }

    #[test]
    fn check_some_squares_queen() {
        let m = QueenMovePatterns::new();

        let starting_square = Square::new(File::B, Rank::R2);
        let expected_squares = vec![
            Square::new(File::A, Rank::R1),
            Square::new(File::C, Rank::R3),
            Square::new(File::D, Rank::R4),
            Square::new(File::E, Rank::R5),
            Square::new(File::F, Rank::R6),
            Square::new(File::G, Rank::R7),
            Square::new(File::H, Rank::R8),
            Square::new(File::C, Rank::R1),
            Square::new(File::A, Rank::R3),
            Square::new(File::B, Rank::R1),
            Square::new(File::B, Rank::R3),
            Square::new(File::B, Rank::R4),
            Square::new(File::B, Rank::R5),
            Square::new(File::B, Rank::R6),
            Square::new(File::B, Rank::R7),
            Square::new(File::B, Rank::R8),
            Square::new(File::A, Rank::R2),
            Square::new(File::C, Rank::R2),
            Square::new(File::D, Rank::R2),
            Square::new(File::E, Rank::R2),
            Square::new(File::F, Rank::R2),
            Square::new(File::G, Rank::R2),
            Square::new(File::H, Rank::R2),
        ];
        let expected_board = Bitboard::from_squares_ref(expected_squares.iter());

        let result = m.get_move(&starting_square, &Bitboard(0));

        assert_eq!(result, expected_board)
    }

    #[test]
    fn get_never_panics_queen() {
        let m = QueenMovePatterns::new();
        for s in Square::ALL.iter() {
            let _res = m.get_move(s, &Bitboard(0)); // should not panic
        }
    }
}
