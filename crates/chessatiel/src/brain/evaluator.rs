use guts::{Color, Position};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CentipawnScore(pub i32);

impl std::fmt::Display for CentipawnScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Centipawn score: {}", self.0)
    }
}

#[derive(Debug)]
pub enum Strategy {
    PieceCount,
}

#[derive(Debug)]
pub struct Evaluator {
    strategy: Strategy,
}

impl Evaluator {
    pub fn new() -> Self {
        Self::with_strategy(Strategy::PieceCount)
    }

    pub fn with_strategy(strategy: Strategy) -> Self {
        Self { strategy }
    }

    pub fn evaluate(&self, position: &Position) -> CentipawnScore {
        match self.strategy {
            Strategy::PieceCount => self.evaluate_piececount(position),
        }
    }

    fn evaluate_piececount(&self, position: &Position) -> CentipawnScore {
        let white_pieces = position.board()[Color::White].all_pieces().count_ones() as i32;
        let black_pieces = position.board()[Color::Black].all_pieces().count_ones() as i32;

        CentipawnScore(100 * (white_pieces - black_pieces))
    }
}

#[cfg(test)]
mod tests {
    mod piececount {
        use super::super::*;
        use std::str::FromStr;

        fn evaluate(position: &Position) -> CentipawnScore {
            let evaluator = Evaluator::with_strategy(Strategy::PieceCount);
            evaluator.evaluate(position)
        }

        #[test]
        fn default_position_equal() {
            let pos = Position::default();
            assert_eq!(evaluate(&pos), CentipawnScore(0))
        }

        #[test]
        fn one_piece_100_score() {
            let pos =
                Position::from_str("rnbqkbnr/pppp1ppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap();
            assert_eq!(evaluate(&pos), CentipawnScore(100));

            let pos =
                Position::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap();
            assert_eq!(evaluate(&pos), CentipawnScore(-100));
        }
    }
}
