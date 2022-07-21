use crate::brain::evaluator::{CentipawnScore, Evaluator, PieceCountEvaluator};
use crate::brain::position_history::PositionHistory;
use crate::brain::{MoveResult, SHARED_COMPONENTS};
use guts::{Move, MoveBuffer};
use log::{info};
use tokio::sync::mpsc;
use tokio::sync::watch;

struct SearchConfig {
    depth: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { depth: 3 }
    }
}

pub struct Searcher<'a, E: Evaluator = PieceCountEvaluator> {
    position_history: &'a mut PositionHistory,
    cancel_rx: watch::Receiver<()>,
    evaluator: E,
    config: SearchConfig,
}

impl<'a> Searcher<'a, PieceCountEvaluator> {
    pub fn new(position_history: &'a mut PositionHistory, cancel_rx: watch::Receiver<()>) -> Self {
        Self::with_evaluator_and_config(
            position_history,
            cancel_rx,
            PieceCountEvaluator::new(),
            SearchConfig::default(),
        )
    }
}

impl<'a, E: Evaluator> Searcher<'a, E> {
    fn with_evaluator_and_config(
        position_history: &'a mut PositionHistory,
        cancel_rx: watch::Receiver<()>,
        evaluator: E,
        config: SearchConfig,
    ) -> Self {
        Self {
            position_history,
            cancel_rx,
            evaluator,
            config,
        }
    }

    pub fn search(&mut self, output: mpsc::UnboundedSender<MoveResult>) {
        match self.do_search(output) {
            Ok(_) => {}
            Err(err) => panic!("Search had an error: {:?}", err),
        }
    }
    fn do_search(&mut self, output: mpsc::UnboundedSender<MoveResult>) -> Result<(), SearchError> {
        info!("Starting search");
        let mut buf = MoveBuffer::new();
        let current_position = self.position_history.current_position();
        let _in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(current_position, &mut buf);

        let mut best_result: Option<MoveResult> = None;

        let pos = current_position.clone();
        for m in buf.into_iter() {
            #[cfg(debug)]
            let ph_len = self.position_history.count();
            let mut pos = pos.clone();
            pos.make_move(m);
            self.position_history.push(pos);

            let mr = self.recurse(self.config.depth, m.clone())?;
            if let Some(br) = &best_result {
                if mr.score() > br.score() {
                    // TODO why does into_iter still need this clone?
                    // mr.push(m.clone());
                    best_result = Some(mr)
                }
            } else {
                best_result = Some(mr);
            }
            let _ = self.position_history.pop();
            #[cfg(debug)]
            debug_assert_eq!(self.position_history.count(), ph_len);
        }

        let _ = output.send(best_result.unwrap());

        Ok(())
    }

    fn recurse(&mut self, depth: usize, m: Move) -> Result<MoveResult, SearchError> {
        self.cancel()?;

        let mut buf = MoveBuffer::new();
        let current_position = self.position_history.current_position();
        let in_check = SHARED_COMPONENTS
            .move_generator
            .generate_legal_moves_for(current_position, &mut buf);
        if buf.is_empty() {
            return Ok(if in_check {
                MoveResult::new(CentipawnScore::CHECKMATED, m)
            } else {
                MoveResult::new(CentipawnScore::ZERO, m)
            });
        };

        if depth == 0 {
            let score = self.evaluator.evaluate(current_position);
            let mr = MoveResult::new(score, m.clone());

            return Ok(mr);
        }

        let mut best_result: Option<MoveResult> = None;

        let pos = current_position.clone();
        for m in buf.into_iter() {
            #[cfg(debug)]
            let ph_len = self.position_history.count();
            let mut pos = pos.clone();
            pos.make_move(m);
            self.position_history.push(pos);
            let mut mr = self.recurse(depth - 1, m.clone())?;
            if let Some(br) = &best_result {
                if mr.score() > -br.score() {
                    // TODO why does into_iter still need this clone?
                    // mr.push(m.clone());
                    mr.invert_score();
                    best_result = Some(mr)
                }
            } else {
                best_result = Some(mr);
            }
            let _ = self.position_history.pop();
            #[cfg(debug)]
            debug_assert_eq!(self.position_history.count(), ph_len);
        }

        Ok(best_result.unwrap())
    }

    #[must_use]
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
    use tokio::test;

    fn get_pc_searcher(
        position_history: &mut PositionHistory,
        cancel_rx: watch::Receiver<()>,
        config: SearchConfig,
    ) -> Searcher {
        Searcher::with_evaluator_and_config(
            position_history,
            cancel_rx,
            PieceCountEvaluator::new(),
            config,
        )
    }

    #[test]
    async fn two_kings_is_draw() {
        let pos = Position::from_str("k7/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let mut history = PositionHistory::new(pos);
        for depth in 0..5 {
            let (cancel_tx, cancel_rx) = watch::channel(());
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut searcher = get_pc_searcher(&mut history, cancel_rx, SearchConfig { depth });
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

    #[test]
    async fn take_the_rook() {
        let pos = Position::from_str("k7/8/8/8/8/8/8/Kr6 w - - 0 1").unwrap();
        let mut history = PositionHistory::new(pos);
        let depth = 1;
        let (cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(&mut history, cancel_rx, SearchConfig { depth });
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

        assert_eq!(mr.first_move().as_uci(), "a1b1");
        assert_eq!(mr.score, CentipawnScore::ZERO);
        drop(cancel_tx);
    }

    #[test]
    async fn after_e2e4() {
        let pos = Position::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
            .unwrap();
        let mut history = PositionHistory::new(pos.clone());
        let (cancel_tx, cancel_rx) = watch::channel(());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut searcher = get_pc_searcher(&mut history, cancel_rx, SearchConfig::default());
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
            buf.moves
        };

        assert!(possible_moves
            .iter()
            .find(|fm| fm.as_uci() == mr.first_move().as_uci())
            .is_some());

        assert_eq!(mr.first_move().as_uci(), "a1b1");
        assert_eq!(mr.score, CentipawnScore::ZERO);
        drop(cancel_tx);
    }
}
