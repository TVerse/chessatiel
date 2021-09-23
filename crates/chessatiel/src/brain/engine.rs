use crate::brain::position_evaluator::PositionEvaluator;
use crate::brain::statistics::Statistics;
use crate::brain::transposition_table::{
    TranspositionTable, TranspositionTableEntry, TranspositionTableResult,
};
use crate::brain::{Centipawn, Score, SearchResult};
use beak::{InfoPayload, OutgoingCommand, ScorePayload};
use guts::{Move, MoveBuffer, MoveGenerator, Position};
use std::cell::RefCell;
use std::sync::mpsc::Sender;
use std::sync::{atomic, Arc};

pub struct Engine {
    move_generator: MoveGenerator,
    position_evaluator: PositionEvaluator,
    statistics: Arc<Statistics>,
    transposition_table: RefCell<TranspositionTable>,
    outgoing_uci: Sender<OutgoingCommand>,
}

impl Engine {
    pub fn new(statistics: Arc<Statistics>, outgoing_uci: Sender<OutgoingCommand>) -> Self {
        Self {
            move_generator: MoveGenerator::default(),
            position_evaluator: PositionEvaluator::default(),
            statistics,
            transposition_table: RefCell::new(TranspositionTable::default()),
            outgoing_uci,
        }
    }

    pub fn move_generator(&self) -> &MoveGenerator {
        &self.move_generator
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

    pub fn reset_tables(&self) {
        self.transposition_table.borrow_mut().clear();
    }

    fn negamax_initial(&self, depth: usize, position: &Position) -> Option<SearchResult> {
        let cached_score = self.transposition_table.borrow().get(position.hash());
        if let Some(cached_score) = &cached_score {
            if cached_score.depth >= depth {
                self.statistics
                    .full_transposition_table_hits()
                    .fetch_add(1, atomic::Ordering::Relaxed);
                return Some(SearchResult::new(
                    cached_score.m.clone(),
                    cached_score.score,
                ));
            }
        }
        let mut buf = MoveBuffer::new();
        let _checked = self
            .move_generator
            .generate_legal_moves_for(position, &mut buf);
        if buf.is_empty() {
            return None;
        }
        let mut overall_best_move = buf.moves[0].clone();
        let mut overall_alpha = Score::new(Centipawn::MIN, None);
        let mut sub_buf = MoveBuffer::new();
        for d in 1..=depth {
            let info_payload = InfoPayload::new().with_depth(d);
            self.outgoing_uci
                .send(OutgoingCommand::Info(info_payload))
                .unwrap();
            let mut alpha = Score::new(Centipawn::MIN, None);
            let beta = Score::new(Centipawn::MAX, Some(0));
            let cached_score = self.transposition_table.borrow().get(position.hash());
            // Check cached move first
            self.swap_cached_move_first(&mut buf, &cached_score);
            let mut best_move = buf.moves[0].clone();
            for m in buf.iter() {
                let new_pos = {
                    let mut p = position.clone();
                    p.make_move(m);
                    p
                };
                let (_m, ri) = self.negamax(d - 1, &new_pos, -beta, -alpha, &mut sub_buf);
                let ri = -ri;
                if ri > alpha {
                    alpha = ri;
                    best_move = m.clone()
                }
            }
            let tt_entry =
                TranspositionTableEntry::new(alpha, d, best_move.clone(), position.hash());
            self.transposition_table.borrow_mut().insert(tt_entry);
            overall_best_move = best_move;
            overall_alpha = alpha;
            let info_payload = InfoPayload::new()
                .with_depth(d)
                .with_pv(vec![overall_best_move.clone()])
                .with_score({
                    if let Some(m) = overall_alpha.mate_depth() {
                        ScorePayload {
                            cp: None,
                            mate: Some(m),
                            bound: None,
                        }
                    } else {
                        ScorePayload {
                            cp: Some(overall_alpha.score().0),
                            mate: None,
                            bound: None,
                        }
                    }
                })
                .with_nodes(
                    self.statistics
                        .nodes_searched()
                        .load(atomic::Ordering::Relaxed),
                );
            self.outgoing_uci
                .send(OutgoingCommand::Info(info_payload))
                .unwrap();
        }
        Some(SearchResult::new(overall_best_move, overall_alpha))
    }

    fn negamax(
        &self,
        depth: usize,
        position: &Position,
        alpha: Score,
        beta: Score,
        buf: &mut MoveBuffer,
    ) -> (Option<Move>, Score) {
        self.statistics
            .nodes_searched()
            .fetch_add(1, atomic::Ordering::Relaxed);
        let cached_score = self.transposition_table.borrow().get(position.hash());
        if let Some(cached_score) = &cached_score {
            if cached_score.depth >= depth {
                self.statistics
                    .full_transposition_table_hits()
                    .fetch_add(1, atomic::Ordering::Relaxed);
                return (Some(cached_score.m.clone()), cached_score.score);
            }
        }
        if depth == 0 {
            (
                None,
                Score::new(self.position_evaluator.evaluate(position), None),
            )
        } else {
            let checked = self.move_generator.generate_legal_moves_for(position, buf);

            if buf.is_empty() {
                // No moves: checkmate or stalemate
                if checked {
                    (None, Score::new(Centipawn::LOSS, Some(-(depth as isize))))
                } else {
                    (None, Score::new(Centipawn::ZERO, None)) // TODO handle explicit draws?
                }
            } else {
                let mut sub_buf = MoveBuffer::new();
                // Check cached move first
                self.swap_cached_move_first(buf, &cached_score);
                let mut alpha = alpha;
                let mut best_move = buf.moves[0].clone();
                for m in buf.iter() {
                    let new_pos = {
                        let mut p = position.clone();
                        p.make_move(m);
                        p
                    };
                    let (_m, score) =
                        self.negamax(depth - 1, &new_pos, -beta, -alpha, &mut sub_buf);
                    let score = -score;
                    if score >= beta {
                        return (Some(best_move), beta);
                    }
                    if score > alpha {
                        alpha = score;
                        best_move = m.clone();
                    }
                }
                let tt_entry =
                    TranspositionTableEntry::new(alpha, depth, best_move.clone(), position.hash());
                self.transposition_table.borrow_mut().insert(tt_entry);
                (Some(best_move), alpha)
            }
        }
    }

    fn swap_cached_move_first(
        &self,
        buf: &mut MoveBuffer,
        cached_score: &Option<TranspositionTableResult>,
    ) {
        if let Some(cached_score) = cached_score {
            self.statistics
                .partial_transposition_table_hits()
                .fetch_add(1, atomic::Ordering::Relaxed);
            let raw_buf = &mut buf.moves;
            let idx = raw_buf.iter().position(|m| {
                // eprintln!("{:?}, {:?}", m.clone(), cached_score.m.clone());
                m.clone() == cached_score.m.clone()
            });
            // None can happen if there's a collision
            if let Some(idx) = idx {
                self.statistics
                    .moves_reordered()
                    .fetch_add(1, atomic::Ordering::Relaxed);
                raw_buf.swap(0, idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::sync::mpsc;

    #[test]
    fn prefer_taking_queen_over_rook() {
        let (tx, _rx) = mpsc::channel();
        let engine = Engine::new(Arc::new(Statistics::default()), tx);

        let position = Position::from_str("k7/8/8/8/2q1r3/3P4/8/K7 w - - 0 1").unwrap();

        let m = engine.search(1, &position);

        assert_eq!(m.unwrap().chess_move().as_uci(), "d3c4")
    }
}
