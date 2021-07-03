use crate::brain::position_evaluator::PositionEvaluator;
use crate::brain::statistics::Statistics;
use crate::brain::transposition_table::{TranspositionTable, TranspositionTableEntry};
use crate::brain::{Centipawn, ResultInfo, SearchResult};
use guts::{MoveBuffer, MoveGenerator, Position};
use std::cell::RefCell;
use std::sync::{atomic, Arc};

#[derive(Default)]
pub struct Engine {
    move_generator: MoveGenerator,
    position_evaluator: PositionEvaluator,
    statistics: Arc<Statistics>,
    transposition_table: RefCell<TranspositionTable>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_generator(&self) -> &MoveGenerator {
        &self.move_generator
    }

    pub fn statistics(&self) -> &Arc<Statistics> {
        &self.statistics
    }

    pub fn perft(&self, depth: usize, position: &Position) -> usize {
        self.move_generator.perft(position, depth)
    }

    pub fn search(&self, depth: usize, position: &Position) -> Option<SearchResult> {
        if depth == 0 {
            return None;
        };
        self.statistics.reset();
        self.negamax_initial(depth, position)
    }

    pub fn reset_tables(&mut self) {
        self.transposition_table.borrow_mut().clear();
    }

    fn negamax_initial(&self, depth: usize, position: &Position) -> Option<SearchResult> {
        let mut buf = MoveBuffer::new();
        let _checked = self
            .move_generator
            .generate_legal_moves_for(position, &mut buf);
        let mut alpha = ResultInfo::new(Centipawn::MIN, None);
        let beta = ResultInfo::new(Centipawn::MAX, Some(0));
        let mut best_result = None;
        let mut sub_buf = MoveBuffer::new();
        for m in buf.iter() {
            let new_pos = {
                let mut p = position.clone();
                p.make_move(&m);
                p
            };
            let ri = -self.negamax(depth - 1, &new_pos, -beta, -alpha, &mut sub_buf);
            if ri >= beta {
                return best_result;
            }
            if ri > alpha {
                alpha = ri;
                best_result = Some(SearchResult::new(m.clone(), ri))
            }
        }
        best_result
    }

    fn negamax(
        &self,
        depth: usize,
        position: &Position,
        alpha: ResultInfo,
        beta: ResultInfo,
        buf: &mut MoveBuffer,
    ) -> ResultInfo {
        self.statistics
            .nodes_searched()
            .fetch_add(1, atomic::Ordering::Relaxed);
        let cached_score = self.transposition_table.borrow().get(position.hash());
        if let Some(cached_score) = cached_score {
            if cached_score.depth >= depth {
                return cached_score.result_info;
            }
        }
        if depth == 0 {
            ResultInfo::new(self.position_evaluator.evaluate(position), None)
        } else {
            let checked = self.move_generator.generate_legal_moves_for(position, buf);
            if buf.is_empty() {
                // No moves: checkmate or stalemate
                if checked {
                    ResultInfo::new(Centipawn::LOSS, Some(-(depth as isize)))
                } else {
                    ResultInfo::new(Centipawn::ZERO, None) // TODO handle explicit draws?
                }
            } else {
                let mut sub_buf = MoveBuffer::new();
                let mut alpha = alpha;
                // TODO move ordering based on TT
                for m in buf.iter() {
                    let new_pos = {
                        let mut p = position.clone();
                        p.make_move(&m);
                        p
                    };
                    let score = -self.negamax(depth - 1, &new_pos, -beta, -alpha, &mut sub_buf);
                    if score >= beta {
                        let tt_entry = TranspositionTableEntry::new(score, depth, position.hash());
                        self.transposition_table.borrow_mut().insert(tt_entry);
                        return beta;
                    }
                    if score > alpha {
                        alpha = score
                    }
                }
                let tt_entry = TranspositionTableEntry::new(alpha, depth, position.hash());
                self.transposition_table.borrow_mut().insert(tt_entry);
                alpha
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn prefer_taking_queen_over_rook() {
        let engine = Engine::new();

        let position = Position::from_str("k7/8/8/8/2q1r3/3P4/8/K7 w - - 0 1").unwrap();

        let m = engine.search(1, &position);

        assert_eq!(m.unwrap().chess_move().as_uci(), "d3c4")
    }
}
