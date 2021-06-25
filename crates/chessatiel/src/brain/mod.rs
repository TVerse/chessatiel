mod info_emitter;
mod position_evaluator;
pub mod statistics;

use crate::brain::position_evaluator::PositionEvaluator;
use crate::brain::statistics::Statistics;
use guts::{Move, MoveBuffer, MoveGenerator, Position};
use std::cmp::Ordering;
use std::ops::Neg;
use std::sync::{atomic, Arc};

#[derive(Debug, Clone)]
pub struct SearchResult {
    chess_move: Move,
    result_info: ResultInfo,
}

impl SearchResult {
    pub fn new(chess_move: Move, result_info: ResultInfo) -> Self {
        Self {
            chess_move,
            result_info,
        }
    }

    pub fn chess_move(&self) -> &Move {
        &self.chess_move
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ResultInfo {
    score: Centipawn,
    mate_depth: Option<isize>,
}

impl ResultInfo {
    pub fn new(score: Centipawn, mate_depth: Option<isize>) -> Self {
        Self { score, mate_depth }
    }
}

impl PartialOrd for ResultInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResultInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then(match (self.mate_depth, other.mate_depth) {
                (Some(s), Some(o)) => s.cmp(&o),
                (Some(s), None) | (None, Some(s)) => s.cmp(&0),
                (None, None) => Ordering::Equal,
            })
    }
}

impl Neg for ResultInfo {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            score: -self.score,
            mate_depth: self.mate_depth.map(|i| -i),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Centipawn(i64);

impl Centipawn {
    pub const ZERO: Centipawn = Centipawn(0);

    pub const WIN: Centipawn = Centipawn(i64::MAX / 2);
    const MAX: Centipawn = Centipawn(i64::MAX);
    pub const LOSS: Centipawn = Centipawn(i64::MIN / 2);
    const MIN: Centipawn = Centipawn(i64::MIN + 1); // to avoid -MIN=MIN
}

impl PartialOrd for Centipawn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Centipawn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Neg for Centipawn {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[derive(Default)]
pub struct Engine {
    move_generator: MoveGenerator,
    position_evaluator: PositionEvaluator,
    statistics: Arc<Statistics>,
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
                for m in buf.iter() {
                    let new_pos = {
                        let mut p = position.clone();
                        p.make_move(&m);
                        p
                    };
                    let score = -self.negamax(depth - 1, &new_pos, -beta, -alpha, &mut sub_buf);
                    if score >= beta {
                        return beta;
                    }
                    if score > alpha {
                        alpha = score
                    }
                }
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
