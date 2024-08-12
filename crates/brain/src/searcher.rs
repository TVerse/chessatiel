use crate::evaluator::{Evaluator, MainEvaluator, ScoreBound};
use crate::position_hash_history::PositionHashHistory;
use crate::priority_buffer::PriorityMoveBuffer;
use crate::statistics::StatisticsHolder;
use crate::transposition_table::{TTEntry, TranspositionTable};
use crate::{CentipawnScore, MoveResult, SHARED_COMPONENTS};
use guts::{Move, MoveType, Position};
use log::{debug, info};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::watch;

#[derive(Default)]
pub struct SearcherConfig {
    pub depth: Option<u16>,
}

#[derive(Debug)]
struct SearchResult {
    move_result: MoveResult,
}

impl SearchResult {
    pub fn new(move_result: MoveResult) -> Self {
        Self { move_result }
    }
}

pub struct Searcher<'a, E: Evaluator> {
    position_hash_history: PositionHashHistory,
    current_position: Position,
    stop_rx: watch::Receiver<()>,
    evaluator: E,
    config: SearcherConfig,
    statistics: &'a StatisticsHolder,
    transposition_table: &'a mut TranspositionTable,
}

impl<'a> Searcher<'a, MainEvaluator<'static>> {
    pub fn new(
        position_and_history: PositionHashHistory,
        current_position: Position,
        stop_rx: watch::Receiver<()>,
        config: SearcherConfig,
        statistics: &'a StatisticsHolder,
        transposition_table: &'a mut TranspositionTable,
    ) -> Self {
        Self::with_evaluator_and_config(
            position_and_history,
            current_position,
            stop_rx,
            Default::default(),
            config,
            statistics,
            transposition_table,
        )
    }
}

impl<'a, E: Evaluator> Searcher<'a, E> {
    pub fn with_evaluator_and_config(
        position_hash_history: PositionHashHistory,
        current_position: Position,
        stop_rx: watch::Receiver<()>,
        evaluator: E,
        config: SearcherConfig,
        statistics: &'a StatisticsHolder,
        transposition_table: &'a mut TranspositionTable,
    ) -> Self {
        Self {
            position_hash_history,
            current_position,
            stop_rx,
            evaluator,
            config,
            statistics,
            transposition_table,
        }
    }

    pub fn search(&mut self, output: mpsc::UnboundedSender<MoveResult>) {
        match self.do_search(output) {
            Ok(_) => info!("Search completed"),
            Err(e) => info!("Search error: {e}"),
        }
    }

    fn do_search(&mut self, output: mpsc::UnboundedSender<MoveResult>) -> Result<(), SearchError> {
        #[cfg(debug_assertions)]
        let original_pos = self.current_position.clone();

        let mut buf = PriorityMoveBuffer::new();
        let max_depth = if let Some(depth) = self.config.depth {
            depth
        } else {
            u16::MAX
        };
        info!("Setting max depth: {max_depth}");
        for depth in 1..=max_depth {
            self.statistics.depth_changed(depth as u64);
            let best: Option<SearchResult> =
                Some(self.recurse(CentipawnScore::MIN, CentipawnScore::MAX, depth, &mut buf)?);

            debug!("Best move: {best:?}");

            if let Some(b) = best {
                output.send(b.move_result).unwrap();
            }

            #[cfg(debug_assertions)]
            debug_assert_eq!(self.current_position, original_pos, "Difference top-level");
        }

        Ok(())
    }

    fn recurse(
        &mut self,
        mut alpha: CentipawnScore,
        mut beta: CentipawnScore,
        depth: u16,
        buf: &mut PriorityMoveBuffer,
    ) -> Result<SearchResult, SearchError> {
        self.stop()?;
        let mut maybe_previously_best_move: Option<&Move> = None;
        if let Some(cached) = self.transposition_table.get(self.current_position.hash()) {
            if cached.hash == self.current_position.hash() {
                self.statistics.tt_hit();
                if cached.depth >= depth {
                    match cached.bound {
                        ScoreBound::Exact => {
                            let mut mr = MoveResult::new(cached.score);
                            mr.push(cached.m.clone().unwrap());
                            return Ok(SearchResult::new(mr));
                        }
                        ScoreBound::Upper => {
                            beta = cached.score;
                        }
                        ScoreBound::Lower => {
                            alpha = cached.score;
                        }
                    }
                }
            }
                maybe_previously_best_move = cached.m.as_ref();
        }

        self.statistics.node_searched();

        if self.position_hash_history.is_threefold_repetition() {
            return Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)));
        }
        if self.current_position.halfmove_clock() >= 50 {
            return Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)));
        }

        let mut new_buf = PriorityMoveBuffer::new();
        if depth == 0 {
            return self.quiescence(-beta, -alpha, &mut new_buf);
        }

        buf.clear();
        let in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(&self.current_position, buf);

        if buf.is_empty() {
            return if in_check {
                debug!("Returning mate");
                Ok(SearchResult::new(MoveResult::new(
                    CentipawnScore::CHECKMATED,
                )))
            } else {
                debug!("Returning draw");
                Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)))
            };
        }

        let mut best_result: SearchResult = SearchResult::new(MoveResult::new(alpha));
        let mut was_alpha_increased = false;
        if let Some(m) = maybe_previously_best_move {
            buf.set_priority(m, u8::MAX);
        }
        while let Some(m) = buf.pop() {
            #[cfg(debug_assertions)]
            let orig_pos = self.current_position.clone();
            #[cfg(debug_assertions)]
            let orig_history = self.position_hash_history.clone();

            self.current_position.make_move(&m);
            self.position_hash_history
                .push(self.current_position.hash());

            let mut new_result = self.recurse(-beta, -alpha, depth - 1, &mut new_buf)?;
            new_result.move_result.invert_score();

            if new_result.move_result.score >= beta {
                debug!(
                    "Got a beta cutoff with beta {beta:?} on move {m}",
                    m = m.as_uci()
                );
                self.transposition_table.set(TTEntry {
                    hash: self.current_position.hash(),
                    depth,
                    score: new_result.move_result.score,
                    bound: ScoreBound::Lower,
                    m: Some(m.clone()),
                });
                self.position_hash_history.pop();
                self.current_position.unmake_move(&m);
                new_result.move_result.push(m.clone());
                return Ok(new_result);
            }

            if new_result.move_result.score > alpha {
                was_alpha_increased = true;
                new_result.move_result.push(m.clone());
                debug!(
                    "Got an alpha update with alpha {alpha:?} with new best move {new_result:?}"
                );
                alpha = new_result.move_result.score;
                best_result = new_result;
            }

            let _ = self.position_hash_history.pop();
            self.current_position.unmake_move(&m);

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                self.position_hash_history, orig_history,
                "Difference during move {m}, original_history: {:?}",
                orig_history
            );
            #[cfg(debug_assertions)]
            debug_assert_eq!(
                self.current_position, orig_pos,
                "Difference during move {m}, original_position: {}",
                orig_pos
            )
        }

        self.transposition_table.set(TTEntry {
            hash: self.current_position.hash(),
            depth,
            score: best_result.move_result.score,
            bound: if was_alpha_increased {
                ScoreBound::Exact
            } else {
                ScoreBound::Upper
            },
            m: best_result.move_result.first_move().cloned(),
        });

        Ok(best_result)
    }

    // TODO figure out a way to merge this with `recurse`, might not be possible
    fn quiescence(
        &mut self,
        mut alpha: CentipawnScore,
        beta: CentipawnScore,
        buf: &mut PriorityMoveBuffer,
    ) -> Result<SearchResult, SearchError> {
        self.statistics.node_searched();
        // Assume we can do better than the current evaluation
        let stand_pat = self.evaluator.evaluate(&self.current_position);
        if stand_pat >= beta {
            return Ok(SearchResult::new(MoveResult::new(beta)));
        }
        if alpha < stand_pat {
            alpha = stand_pat
        }
        if self.position_hash_history.is_threefold_repetition() {
            return Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)));
        }
        if self.current_position.halfmove_clock() >= 50 {
            return Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)));
        }
        buf.clear();
        let in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(&self.current_position, buf);
        if buf.is_empty() {
            return if in_check {
                debug!("Returning mate");
                Ok(SearchResult::new(MoveResult::new(
                    CentipawnScore::CHECKMATED,
                )))
            } else {
                debug!("Returning draw");
                Ok(SearchResult::new(MoveResult::new(CentipawnScore::ZERO)))
            };
        }
        let mut best_result: SearchResult = SearchResult::new(MoveResult::new(alpha));
        let mut new_buf = PriorityMoveBuffer::new();
        for m in buf.unordered_iter() {
            if m.move_type().contains(MoveType::CAPTURE) {
                #[cfg(debug_assertions)]
                let orig_pos = self.current_position.clone();
                #[cfg(debug_assertions)]
                let orig_history = self.position_hash_history.clone();
                self.current_position.make_move(m);
                self.position_hash_history
                    .push(self.current_position.hash());

                let mut new_result = self.quiescence(-beta, -alpha, &mut new_buf)?;
                new_result.move_result.invert_score();

                if new_result.move_result.score >= beta {
                    debug!(
                        "Got a beta cutoff with beta {beta:?} on move {m}",
                        m = m.as_uci()
                    );
                    self.position_hash_history.pop();
                    self.current_position.unmake_move(m);
                    new_result.move_result.push(m.clone());
                    return Ok(new_result);
                }

                if new_result.move_result.score > alpha {
                    new_result.move_result.push(m.clone());
                    debug!(
                        "Got an alpha update with alpha {alpha:?} with new best move{new_result:?}"
                    );
                    alpha = new_result.move_result.score;
                    best_result = new_result;
                }

                let _ = self.position_hash_history.pop();
                self.current_position.unmake_move(m);

                #[cfg(debug_assertions)]
                debug_assert_eq!(
                    self.position_hash_history, orig_history,
                    "Difference during move {m}, original_history: {:?}",
                    orig_history
                );
                #[cfg(debug_assertions)]
                debug_assert_eq!(
                    self.current_position, orig_pos,
                    "Difference during move {m}, original_position: {}",
                    orig_pos
                )
            }
        }
        Ok(best_result)
    }

    fn stop(&mut self) -> Result<(), SearchError> {
        match self.stop_rx.has_changed() {
            Ok(false) => Ok(()),
            _ => {
                info!("Searcher received stop");
                Err(SearchError::Stopped)
            }
        }
    }
}

#[derive(Debug, Error)]
enum SearchError {
    #[error("search was stopped")]
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::PieceCountEvaluator;
    use guts::{BasicMoveBuffer, MoveGenerator, Position};
    use std::str::FromStr;

    fn get_pc_searcher<'a>(
        history: PositionHashHistory,
        position: Position,
        stop_rx: watch::Receiver<()>,
        config: SearcherConfig,
        stats: &'a StatisticsHolder,
        tt: &'a mut TranspositionTable,
    ) -> Searcher<'a, PieceCountEvaluator> {
        Searcher::with_evaluator_and_config(
            history,
            position,
            stop_rx,
            Default::default(),
            config,
            stats,
            tt,
        )
    }

    #[tokio::test]
    async fn two_kings_is_draw() {
        for depth in 1..5 {
            let stats = StatisticsHolder::new();
            let mut tt = TranspositionTable::default();
            let pos = Position::from_str("k7/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
            let history = PositionHashHistory::new(pos.hash());
            let (stop_tx, stop_rx) = watch::channel(());
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut searcher = get_pc_searcher(
                history,
                pos,
                stop_rx,
                SearcherConfig { depth: Some(depth) },
                &stats,
                &mut tt,
            );
            searcher.search(tx);

            let last = {
                let mut tmp = None;
                let mut ctr = 0;
                while let Some(r) = rx.recv().await {
                    tmp = Some(r);
                    ctr += 1;
                }
                assert!(ctr > 0, "{ctr}");
                tmp.unwrap()
            };

            assert_eq!(
                last.score(),
                CentipawnScore::ZERO,
                "Depth {depth}: expected {expected:?}, but got {actual:?}",
                expected = CentipawnScore::ZERO,
                actual = last.score()
            );
            drop(stop_tx);
        }
    }

    // TODO relies on checkmate, need a mate-in metric first
    // #[ignore]
    // async fn take_the_rook() {
    //     let stats = StatisticsHolder::new();
    //     let mut tt = TranspositionTable::new();
    //     let pos = Position::from_str("k7/8/8/8/8/8/8/Kr6 w - - 0 1").unwrap();
    //     let history = PositionHashHistory::new(pos.hash());
    //     let depth = 3;
    //     let (_stop_tx, stop_rx) = watch::channel(());
    //     let (tx, mut rx) = mpsc::unbounded_channel();
    //     let mut searcher = get_pc_searcher(
    //         history,
    //         pos,
    //         stop_rx,
    //         SearcherConfig { depth: Some(depth) },
    //         &stats,
    //         &mut tt,
    //     );
    //     searcher.search(tx);
    //
    //     let mr = {
    //         let mut tmp = None;
    //         let mut ctr = 0;
    //         while let Some(r) = rx.recv().await {
    //             tmp = Some(r);
    //             ctr += 1;
    //         }
    //         assert!(ctr > 0);
    //         tmp.unwrap()
    //     };
    //
    //     assert_eq!(mr.first_move().unwrap().as_uci(), "a1b1");
    //     assert_eq!(mr.score, CentipawnScore::ZERO);
    // }
    #[tokio::test]
    async fn take_the_pawn() {
        let stats = StatisticsHolder::new();
        let mut tt = TranspositionTable::default();
        let pos = Position::from_str("k7/8/8/8/8/8/2p5/K7 w - - 0 1").unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let depth = 3;
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(
            history,
            pos,
            stop_rx,
            SearcherConfig { depth: Some(depth) },
            &stats,
            &mut tt,
        );
        searcher.search(tx);

        let mr = {
            let mut tmp = None;
            let mut ctr = 0;
            while let Some(r) = rx.recv().await {
                tmp = Some(r);
                ctr += 1;
            }
            assert!(ctr > 0);
            tmp.unwrap()
        };

        assert_eq!(mr.first_move().unwrap().as_uci(), "a1b2");
        assert_eq!(mr.score, CentipawnScore::ZERO);
    }

    #[tokio::test]
    async fn illegal_move_after_after_e2e4() {
        let stats = StatisticsHolder::new();
        let mut tt = TranspositionTable::default();
        let pos = Position::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
            .unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let (stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(
            history,
            pos.clone(),
            stop_rx,
            SearcherConfig { depth: Some(4) },
            &stats,
            &mut tt,
        );
        searcher.search(tx);

        let mr = {
            let mut tmp = None;
            let mut ctr = 0;
            while let Some(r) = rx.recv().await {
                tmp = Some(r);
                ctr += 1;
            }
            assert!(ctr > 0);
            tmp.unwrap()
        };

        let possible_moves = {
            let mut buf = BasicMoveBuffer::new();
            let _in_check = MoveGenerator::new().generate_legal_moves_for(&pos, &mut buf);
            buf
        };

        assert!(
            possible_moves
                .iter()
                .any(|fm| fm.as_uci() == mr.first_move().unwrap().as_uci()),
            "{:?}",
            mr
        );

        drop(stop_tx);
    }

    #[tokio::test]
    async fn should_not_play_capturable_pawn() {
        let stats = StatisticsHolder::new();
        let mut tt = TranspositionTable::default();
        let pos =
            Position::from_str("rnbqkbnr/2pppppp/1p6/p7/3PP3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(
            history,
            pos,
            stop_rx,
            SearcherConfig { depth: Some(4) },
            &stats,
            &mut tt,
        );
        searcher.search(tx);

        let mr = {
            let mut tmp = None;
            let mut ctr = 0;
            while let Some(r) = rx.recv().await {
                tmp = Some(r);
                ctr += 1;
            }
            assert!(ctr > 0);
            tmp.unwrap()
        };

        assert!(mr.score >= CentipawnScore::ZERO, "{:?}", mr.score);
        assert_ne!(mr.first_move().unwrap().as_uci(), "b6b5");
    }

    #[tokio::test]
    async fn give_checkmate() {
        let stats = StatisticsHolder::new();
        let mut tt = TranspositionTable::default();
        let pos = Position::from_str("8/8/k1K5/8/8/8/8/1R6 w - - 0 1").unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let depth = 4;
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(
            history,
            pos,
            stop_rx,
            SearcherConfig { depth: Some(depth) },
            &stats,
            &mut tt,
        );
        searcher.search(tx);

        let mr = {
            let mut tmp = None;
            let mut ctr = 0;
            while let Some(r) = rx.recv().await {
                tmp = Some(r);
                ctr += 1;
            }
            assert!(ctr > 0);
            tmp.unwrap()
        };

        assert_eq!(mr.first_move().unwrap().as_uci(), "b1a1");
        assert_eq!(mr.score, -CentipawnScore::CHECKMATED);
    }
}
