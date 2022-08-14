use crate::evaluator::{Evaluator, PieceCountEvaluator};
use crate::position_hash_history::PositionHashHistory;
use crate::{CentipawnScore, MoveResult, SHARED_COMPONENTS};
use guts::{MoveBuffer, Position};
use log::info;
use tokio::sync::mpsc;
use tokio::sync::watch;

pub struct SearchConfig {
    pub depth: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { depth: 4 }
    }
}

pub struct Searcher<'a, E: Evaluator = PieceCountEvaluator> {
    position_hash_history: &'a mut PositionHashHistory,
    current_position: &'a mut Position,
    cancel_rx: watch::Receiver<()>,
    evaluator: E,
    config: SearchConfig,
}

impl<'a> Searcher<'a, PieceCountEvaluator> {
    pub fn new(
        position_and_history: &'a mut PositionHashHistory,
        current_position: &'a mut Position,
        cancel_rx: watch::Receiver<()>,
    ) -> Self {
        Self::with_evaluator_and_config(
            position_and_history,
            current_position,
            cancel_rx,
            PieceCountEvaluator::new(),
            SearchConfig::default(),
        )
    }
}

impl<'a, E: Evaluator> Searcher<'a, E> {
    pub fn with_evaluator_and_config(
        position_and_history: &'a mut PositionHashHistory,
        current_position: &'a mut Position,
        cancel_rx: watch::Receiver<()>,
        evaluator: E,
        config: SearchConfig,
    ) -> Self {
        Self {
            position_hash_history: position_and_history,
            current_position,
            cancel_rx,
            evaluator,
            config,
        }
    }

    pub fn search(&mut self, output: mpsc::UnboundedSender<MoveResult>) {
        let _ = self.do_search(output);
    }

    fn do_search(&mut self, output: mpsc::UnboundedSender<MoveResult>) -> Result<(), SearchError> {
        #[cfg(debug_assertions)]
        let original_pos = self.current_position.clone();

        let mut buf = MoveBuffer::new();
        let best: Option<MoveResult> = Some(self.recurse(self.config.depth, &mut buf)?);

        if let Some(b) = best {
            output.send(b).unwrap();
        }

        #[cfg(debug_assertions)]
        debug_assert_eq!(*self.current_position, original_pos, "Difference top-level");

        Ok(())
    }

    fn recurse(&mut self, depth: usize, buf: &mut MoveBuffer) -> Result<MoveResult, SearchError> {
        self.cancel()?;
        buf.clear();

        if depth == 0 {
            let score = self.evaluator.evaluate(self.current_position);
            return Ok(MoveResult::new(score));
        }

        let in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(self.current_position, buf);

        if buf.is_empty() {
            return if in_check {
                Ok(MoveResult::new(CentipawnScore::CHECKMATED))
            } else {
                Ok(MoveResult::new(CentipawnScore::ZERO))
            };
        }

        let mut best_move: Option<MoveResult> = None;
        let mut new_buf = MoveBuffer::new();
        for m in buf.iter() {
            #[cfg(debug_assertions)]
            let orig_pos = self.current_position.clone();

            self.current_position.make_move(m);
            self.position_hash_history
                .push(self.current_position.hash());

            let mut new_result = self.recurse(depth - 1, &mut new_buf)?;
            new_result.invert_score();
            if let Some(bm) = &best_move {
                if new_result.score > bm.score {
                    new_result.push(m.clone());
                    best_move = Some(new_result)
                }
            } else {
                new_result.push(m.clone());
                best_move = Some(new_result);
            }

            let _ = self.position_hash_history.pop();
            self.current_position.unmake_move(m);

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                *self.current_position, orig_pos,
                "Difference during move {m}, original_position: {}",
                orig_pos
            )
        }

        Ok(best_move.expect("at least one legal move"))
    }

    fn cancel(&mut self) -> Result<(), SearchError> {
        match self.cancel_rx.has_changed() {
            Ok(false) => Ok(()),
            _ => {
                info!("Searcher received cancellation");
                Err(SearchError::Cancelled)
            }
        }
    }
}

#[derive(Debug)]
enum SearchError {
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use guts::{MoveGenerator, Position};
    use std::str::FromStr;

    fn get_pc_searcher<'a>(
        history: &'a mut PositionHashHistory,
        position: &'a mut Position,
        cancel_rx: watch::Receiver<()>,
        config: SearchConfig,
    ) -> Searcher<'a> {
        Searcher::with_evaluator_and_config(
            history,
            position,
            cancel_rx,
            PieceCountEvaluator::new(),
            config,
        )
    }

    #[tokio::test]
    async fn two_kings_is_draw() {
        let mut pos = Position::from_str("k7/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        for depth in 0..5 {
            let (cancel_tx, cancel_rx) = watch::channel(());
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut searcher =
                get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth });
            searcher.search(tx);

            let last = {
                let mut tmp = None;
                let mut ctr = 0;
                while let Some(r) = rx.recv().await {
                    tmp = Some(r);
                    ctr += 1;
                }
                assert!(ctr > 0);
                tmp.unwrap()
            };

            assert_eq!(
                last.score(),
                CentipawnScore::ZERO,
                "Depth {depth}: expected {expected:?}, but got {actual:?}",
                expected = CentipawnScore::ZERO,
                actual = last.score()
            );
            drop(cancel_tx);
        }
    }

    #[tokio::test]
    async fn take_the_rook() {
        let mut pos = Position::from_str("k7/8/8/8/8/8/8/Kr6 w - - 0 1").unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        let depth = 3;
        let (_cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth });
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
        let mut pos = Position::from_str("k7/8/8/8/8/8/2p5/K7 w - - 0 1").unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        let depth = 3;
        let (_cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth });
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
        let mut pos =
            Position::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
                .unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        let (cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth: 4 });
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

        drop(cancel_tx);
    }

    #[tokio::test]
    async fn should_not_play_capturable_pawn() {
        let mut pos =
            Position::from_str("rnbqkbnr/2pppppp/1p6/p7/3PP3/2N2N2/PPP2PPP/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        let (_cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth: 4 });
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
        let mut pos = Position::from_str("8/8/k1K5/8/8/8/8/1R6 w - - 0 1").unwrap();
        let mut history = PositionHashHistory::new(pos.hash());
        let depth = 4;
        let (_cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher =
            get_pc_searcher(&mut history, &mut pos, cancel_rx, SearchConfig { depth });
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
