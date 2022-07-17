use crate::brain::evaluator::Evaluator;
use crate::brain::position_manager::PositionHistory;
use crate::brain::{MoveResult, SHARED_COMPONENTS};
use guts::{Color, MoveBuffer};
use log::debug;

#[derive(Debug)]
pub struct Searcher {
    evaluator: Evaluator,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }

    pub fn search(&self, position_history: &PositionHistory) -> MoveResult {
        let cur_pos = position_history.current_position();
        let mut buffer = MoveBuffer::new();
        let _in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(cur_pos, &mut buffer);

        let max = buffer
            .moves
            .into_iter()
            .map(|m| {
                let new_pos = {
                    let mut tmp = cur_pos.clone();
                    tmp.make_move(&m);
                    tmp
                };
                let score = self.evaluator.evaluate(&new_pos);
                debug!("Evaluated move {m} to score {score}");
                (m, score)
            })
            .max_by_key(|p| {
                if cur_pos.active_color() == Color::White {
                    p.1 .0
                } else {
                    -p.1 .0
                }
            });

        if let Some((m, score)) = max {
            MoveResult::BestMove {
                chess_move: m,
                score,
            }
        } else {
            MoveResult::GameAlreadyFinished
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::evaluator::Strategy;
    use guts::Position;
    use std::str::FromStr;

    fn with_piece_count_evaluator() -> Searcher {
        Searcher {
            evaluator: Evaluator::with_strategy(Strategy::PieceCount),
        }
    }

    #[test]
    fn ensure_color_is_correct_white() {
        let searcher = with_piece_count_evaluator();
        let pos = Position::from_str("k7/8/4r3/8/8/4R3/8/K7 w - - 0 1").unwrap();
        let history = PositionHistory::new(pos);
        let result = searcher.search(&history);

        match result {
            MoveResult::BestMove { chess_move, .. } => {
                assert_eq!(chess_move.as_uci(), "e3e6")
            }
            MoveResult::GameAlreadyFinished => panic!(),
        }
    }

    #[test]
    fn ensure_color_is_correct_black() {
        let searcher = with_piece_count_evaluator();
        let pos = Position::from_str("k7/8/4r3/8/8/4R3/8/K7 b - - 0 1").unwrap();
        let history = PositionHistory::new(pos);
        let result = searcher.search(&history);

        match result {
            MoveResult::BestMove { chess_move, .. } => {
                assert_eq!(chess_move.as_uci(), "e6e3")
            }
            MoveResult::GameAlreadyFinished => panic!(),
        }
    }
}
