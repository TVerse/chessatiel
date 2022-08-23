use crate::evaluator::{Evaluator, PieceValueEvaluator};
use crate::position_hash_history::PositionHashHistory;
use crate::statistics::StatisticsHolder;
use crate::{CentipawnScore, MoveResult, SHARED_COMPONENTS};
use guts::{MoveBuffer, Position};
use log::{debug, info};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::watch;

#[derive(Default)]
pub struct SearcherConfig {
    pub depth: Option<usize>,
}

pub struct Searcher<E: Evaluator> {
    position_hash_history: PositionHashHistory,
    current_position: Position,
    stop_rx: watch::Receiver<()>,
    evaluator: E,
    config: SearcherConfig,
    statistics: Arc<StatisticsHolder>,
}

impl Searcher<PieceValueEvaluator> {
    pub fn new(
        position_and_history: PositionHashHistory,
        current_position: Position,
        stop_rx: watch::Receiver<()>,
        config: SearcherConfig,
        statistics: Arc<StatisticsHolder>,
    ) -> Self {
        Self::with_evaluator_and_config(
            position_and_history,
            current_position,
            stop_rx,
            Default::default(),
            config,
            statistics,
        )
    }
}

impl<E: Evaluator> Searcher<E> {
    pub fn with_evaluator_and_config(
        position_hash_history: PositionHashHistory,
        current_position: Position,
        stop_rx: watch::Receiver<()>,
        evaluator: E,
        config: SearcherConfig,
        statistics: Arc<StatisticsHolder>,
    ) -> Self {
        Self {
            position_hash_history,
            current_position,
            stop_rx,
            evaluator,
            config,
            statistics,
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

        let mut buf = MoveBuffer::new();
        let max_depth = if let Some(depth) = self.config.depth {
            depth
        } else {
            usize::MAX
        };
        info!("Setting max depth: {max_depth}");
        for depth in 1..=max_depth {
            self.statistics.depth_changed(depth as u64);
            let best: Option<MoveResult> =
                Some(self.recurse(CentipawnScore::MIN, CentipawnScore::MAX, depth, &mut buf)?);

            debug!("Best move: {best:?}");

            if let Some(b) = best {
                output.send(b).unwrap();
            }

            #[cfg(debug_assertions)]
            debug_assert_eq!(self.current_position, original_pos, "Difference top-level");
        }

        Ok(())
    }

    fn recurse(
        &mut self,
        mut alpha: CentipawnScore,
        beta: CentipawnScore,
        depth: usize,
        buf: &mut MoveBuffer,
    ) -> Result<MoveResult, SearchError> {
        self.stop()?;
        buf.clear();
        self.statistics.node_searched();

        if self.position_hash_history.is_threefold_repetition() {
            return Ok(MoveResult::new(CentipawnScore::ZERO));
        }
        if self.current_position.halfmove_clock() >= 50 {
            return Ok(MoveResult::new(CentipawnScore::ZERO));
        }

        if depth == 0 {
            let score = self.evaluator.evaluate(&self.current_position);
            return Ok(MoveResult::new(score));
        }

        let in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(&self.current_position, buf);

        if buf.is_empty() {
            return if in_check {
                debug!("Returning mate");
                Ok(MoveResult::new(CentipawnScore::CHECKMATED))
            } else {
                debug!("Returning draw");
                Ok(MoveResult::new(CentipawnScore::ZERO))
            };
        }

        let mut best_result: MoveResult = MoveResult::new(alpha);
        let mut new_buf = MoveBuffer::new();
        for m in buf.iter() {
            #[cfg(debug_assertions)]
            let orig_pos = self.current_position.clone();
            #[cfg(debug_assertions)]
            let orig_history = self.position_hash_history.clone();

            self.current_position.make_move(m);
            self.position_hash_history
                .push(self.current_position.hash());

            let mut new_result = self.recurse(-beta, -alpha, depth - 1, &mut new_buf)?;
            new_result.invert_score();

            if new_result.score >= beta {
                debug!(
                    "Got a beta cutoff with beta {beta:?} on move {m}",
                    m = m.as_uci()
                );
                self.position_hash_history.pop();
                self.current_position.unmake_move(m);
                new_result.push(m.clone());
                return Ok(new_result);
            }

            if new_result.score > alpha {
                new_result.push(m.clone());
                debug!("Got an alpha update with alpha {alpha:?} with new best move{new_result:?}");
                alpha = new_result.score;
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
    use guts::{MoveGenerator, Position};
    use std::str::FromStr;

    fn get_pc_searcher(
        history: PositionHashHistory,
        position: Position,
        stop_rx: watch::Receiver<()>,
        config: SearcherConfig,
    ) -> Searcher<PieceCountEvaluator> {
        Searcher::with_evaluator_and_config(
            history,
            position,
            stop_rx,
            Default::default(),
            config,
            Arc::new(StatisticsHolder::new()),
        )
    }

    #[tokio::test]
    async fn two_kings_is_draw() {
        for depth in 1..5 {
            let pos = Position::from_str("k7/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
            let history = PositionHashHistory::new(pos.hash());
            let (stop_tx, stop_rx) = watch::channel(());
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut searcher =
                get_pc_searcher(history, pos, stop_rx, SearcherConfig { depth: Some(depth) });
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

    #[tokio::test]
    async fn take_the_rook() {
        let pos = Position::from_str("k7/8/8/8/8/8/8/Kr6 w - - 0 1").unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let depth = 3;
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(history, pos, stop_rx, SearcherConfig { depth: Some(depth) });
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

        assert_eq!(mr.first_move().unwrap().as_uci(), "a1b1");
        assert_eq!(mr.score, CentipawnScore::ZERO);
    }

    #[tokio::test]
    async fn take_the_pawn() {
        let pos = Position::from_str("k7/8/8/8/8/8/2p5/K7 w - - 0 1").unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let depth = 3;
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(history, pos, stop_rx, SearcherConfig { depth: Some(depth) });
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
            let mut buf = MoveBuffer::new();
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
        let pos =
            Position::from_str("rnbqkbnr/2pppppp/1p6/p7/3PP3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(history, pos, stop_rx, SearcherConfig { depth: Some(4) });
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
        let pos = Position::from_str("8/8/k1K5/8/8/8/8/1R6 w - - 0 1").unwrap();
        let history = PositionHashHistory::new(pos.hash());
        let depth = 4;
        let (_stop_tx, stop_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(history, pos, stop_rx, SearcherConfig { depth: Some(depth) });
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
