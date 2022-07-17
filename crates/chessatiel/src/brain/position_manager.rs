#[cfg(test)]
use guts::Move;
use guts::{MoveBuffer, MoveGenerator, Position};

#[derive(Debug, PartialEq)]
pub struct PositionHistory {
    positions: Vec<Position>,
}

impl Default for PositionHistory {
    fn default() -> Self {
        Self {
            positions: vec![Position::default()],
        }
    }
}

impl PositionHistory {
    pub fn new(initial_position: Position) -> Self {
        Self {
            positions: vec![initial_position],
        }
    }

    #[cfg(test)]
    pub fn set_moves(&mut self, moves: &[Move]) {
        let mut positions = vec![self.positions[0].clone()];
        positions.extend(moves.iter().scan(self.positions[0].clone(), |p, m| {
            p.make_move(m);

            Some(p.clone())
        }));

        self.positions = positions;
    }

    pub fn set_moves_from_strings(&mut self, moves: &[String], move_generator: &MoveGenerator) {
        let mut positions = vec![self.positions[0].clone()];
        let mut buf = MoveBuffer::new();
        positions.extend(moves.iter().scan(self.positions[0].clone(), |p, m| {
            let _ = move_generator.generate_legal_moves_for(p, &mut buf);

            let found_move = buf
                .moves
                .iter()
                .find(|fm| &fm.as_uci() == m)
                .unwrap_or_else(|| panic!("Got invalid move {m}"));

            p.make_move(found_move);

            Some(p.clone())
        }));

        self.positions = positions;
    }

    pub fn current_position(&self) -> &Position {
        self.positions.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guts::{File, MoveType, Piece, Rank, Square};
    use std::str::FromStr;

    #[test]
    fn test_position_history() {
        let mut ph =
            PositionHistory::new(Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap());
        let moves = [
            Move::new(
                Square::new(File::B, Rank::R2),
                Square::new(File::B, Rank::R3),
                Piece::King,
                MoveType::PUSH,
                None,
            ),
            Move::new(
                Square::new(File::D, Rank::R7),
                Square::new(File::E, Rank::R6),
                Piece::King,
                MoveType::PUSH,
                None,
            ),
        ];
        ph.set_moves(&moves);
        let expected = PositionHistory {
            positions: vec![
                Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
                Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap(),
                Position::from_str("8/8/4k3/8/8/1K6/8/8 w - - 2 2").unwrap(),
            ],
        };

        assert_eq!(ph, expected);
        let moves = [Move::new(
            Square::new(File::B, Rank::R2),
            Square::new(File::B, Rank::R3),
            Piece::King,
            MoveType::PUSH,
            None,
        )];
        ph.set_moves(&moves);
        let expected = PositionHistory {
            positions: vec![
                Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
                Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap(),
            ],
        };
        assert_eq!(ph, expected);
    }
}
