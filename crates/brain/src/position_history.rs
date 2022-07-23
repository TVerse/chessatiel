use guts::{MoveBuffer, MoveGenerator, Position};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PositionHistory {
    starting_position: Position,
    positions: Vec<Position>,
}

impl PositionHistory {
    const INITIAL_VEC_CAPACITY: usize = 100;

    pub fn new(starting_position: Position) -> Self {
        Self {
            starting_position,
            positions: Vec::with_capacity(Self::INITIAL_VEC_CAPACITY),
        }
    }

    pub fn reset_with(&mut self, starting_position: Position) {
        self.starting_position = starting_position;
        self.positions = Vec::with_capacity(Self::INITIAL_VEC_CAPACITY);
    }

    pub fn set_moves_from_strings(&mut self, moves: &[String], move_generator: &MoveGenerator) {
        let mut buf = MoveBuffer::new();
        self.positions = moves
            .iter()
            .scan(self.starting_position.clone(), |p, m| {
                let _ = move_generator.generate_legal_moves_for(p, &mut buf);

                let found_move = buf
                    .iter()
                    .find(|fm| &fm.as_uci() == m)
                    .unwrap_or_else(|| panic!("Got invalid move {m}"));

                p.make_move(found_move);

                Some(p.clone())
            })
            .collect();
    }

    pub fn current_position(&self) -> &Position {
        self.positions.last().unwrap_or(&self.starting_position)
    }

    pub fn push(&mut self, position: Position) {
        self.positions.push(position)
    }

    pub fn pop(&mut self) -> Option<Position> {
        self.positions.pop()
    }

    #[cfg(debug_assertions)]
    pub fn count(&self) -> usize {
        1 + self.positions.len()
    }

    #[cfg(test)]
    pub fn is_threefold_repetition(&self) -> bool {
        std::iter::once(&self.starting_position)
            .chain(self.positions.iter())
            .rev()
            .fold(0, |count, p| {
                if p.repetition_compare(self.current_position()) {
                    count + 1
                } else {
                    count
                }
            })
            >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_position_history() {
        let move_generator = MoveGenerator::new();

        let mut ph =
            PositionHistory::new(Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap());
        let moves = ["b2b3".to_string(), "d7e6".to_string()];
        ph.set_moves_from_strings(&moves, &move_generator);
        let expected = PositionHistory {
            starting_position: Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
            positions: vec![
                Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap(),
                Position::from_str("8/8/4k3/8/8/1K6/8/8 w - - 2 2").unwrap(),
            ],
        };

        assert_eq!(ph, expected);
        let moves = ["b2b3".to_string()];
        ph.set_moves_from_strings(&moves, &move_generator);
        let expected = PositionHistory {
            starting_position: Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
            positions: vec![Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap()],
        };
        assert_eq!(ph, expected);
    }

    #[test]
    fn test_threefold() {
        let mut ph = PositionHistory::new(Position::default());
        ph.push(Position::default());
        ph.push(Position::default());

        assert!(ph.is_threefold_repetition());

        let _ = ph.pop();
        assert!(!ph.is_threefold_repetition());
    }

    #[test]
    fn test_not_threefold() {
        let mut ph = PositionHistory::new(Position::default());
        ph.push(Position::default());
        ph.push(Position::from_str("k7/8/4r3/8/8/4R3/8/K7 w - - 0 1").unwrap());

        assert!(!ph.is_threefold_repetition());
    }
}
