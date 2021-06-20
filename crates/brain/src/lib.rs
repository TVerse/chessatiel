mod position_evaluator;
pub mod statistics;

use crate::position_evaluator::PositionEvaluator;
use guts::{Move, MoveBuffer, MoveGenerator, Position};
use std::cmp::Ordering;
use std::ops::Neg;
use crate::statistics::Statistics;
use std::sync::{atomic, Arc};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RelativePlayer {
    Me,
    Opponent,
}

impl PartialOrd for RelativePlayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RelativePlayer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            RelativePlayer::Me => match other {
                RelativePlayer::Me => Ordering::Equal,
                RelativePlayer::Opponent => Ordering::Greater,
            },
            RelativePlayer::Opponent => match other {
                RelativePlayer::Me => Ordering::Less,
                RelativePlayer::Opponent => Ordering::Equal,
            },
        }
    }
}

impl Neg for RelativePlayer {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            RelativePlayer::Me => RelativePlayer::Opponent,
            RelativePlayer::Opponent => RelativePlayer::Me,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FinishState {
    Win(RelativePlayer, usize),
    Draw,
}

impl PartialOrd for FinishState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FinishState {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            FinishState::Win(s, s_at_depth) => match other {
                FinishState::Win(o, o_at_depth) => s.cmp(o).then(s_at_depth.cmp(o_at_depth)),
                FinishState::Draw => match s {
                    RelativePlayer::Me => Ordering::Greater,
                    RelativePlayer::Opponent => Ordering::Less,
                },
            },
            FinishState::Draw => match other {
                FinishState::Win(o, _) => match o {
                    RelativePlayer::Me => Ordering::Greater,
                    RelativePlayer::Opponent => Ordering::Less,
                },
                FinishState::Draw => Ordering::Equal,
            },
        }
    }
}

impl Neg for FinishState {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            FinishState::Win(p, at_depth) => FinishState::Win(-p, at_depth),
            FinishState::Draw => FinishState::Draw,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum GameStatus {
    Playing(Centipawn),
    Finished(FinishState),
}

impl PartialOrd for GameStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            GameStatus::Playing(c) => match other {
                GameStatus::Playing(o) => c.cmp(o),
                GameStatus::Finished(f) => match f {
                    FinishState::Win(c, _) => match c {
                        RelativePlayer::Me => Ordering::Less,
                        RelativePlayer::Opponent => Ordering::Greater,
                    },
                    FinishState::Draw => c.cmp(&Centipawn::ZERO),
                },
            },
            GameStatus::Finished(f) => match f {
                FinishState::Win(c, s_d) => match other {
                    GameStatus::Finished(o_f) => match o_f {
                        FinishState::Win(o_c, o_d) => {
                            if c == o_c {
                                s_d.cmp(o_d)
                            } else {
                                match c {
                                    RelativePlayer::Me => Ordering::Greater,
                                    RelativePlayer::Opponent => Ordering::Less,
                                }
                            }
                        }
                        _ => match c {
                            RelativePlayer::Me => Ordering::Greater,
                            RelativePlayer::Opponent => Ordering::Less,
                        },
                    },
                    _ => match c {
                        RelativePlayer::Me => Ordering::Greater,
                        RelativePlayer::Opponent => Ordering::Less,
                    },
                },
                FinishState::Draw => GameStatus::Playing(Centipawn::ZERO).cmp(other),
            },
        }
    }
}

impl Neg for GameStatus {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            GameStatus::Playing(c) => GameStatus::Playing(-c),
            GameStatus::Finished(f) => GameStatus::Finished(-f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Centipawn(f64);

impl Centipawn {
    pub const ZERO: Centipawn = Centipawn(0.0);
}

impl Eq for Centipawn {}

impl PartialOrd for Centipawn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Centipawn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
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

    pub fn find_move(&self, depth: usize, position: &Position) -> Option<Move> {
        let mut buf = MoveBuffer::new();
        let _checked = self
            .move_generator
            .generate_legal_moves_for(position, &mut buf);

        buf.iter()
            .max_by_key(|m| {
                let new_pos = {
                    let mut p = position.clone();
                    p.make_move(m);
                    p
                };
                let mut buf = MoveBuffer::new();
                let score = -self.negamax(depth, &new_pos, &mut buf);
                score
            })
            .cloned()
    }

    fn negamax(&self, depth: usize, position: &Position, buf: &mut MoveBuffer) -> GameStatus {
        self.statistics.nodes_searched().fetch_add(1, atomic::Ordering::Relaxed);
        if depth == 0 {
            GameStatus::Playing(self.position_evaluator.evaluate(position))
        } else {
            let checked = self.move_generator.generate_legal_moves_for(position, buf);
            buf.iter()
                .map(|m| {
                    let new_pos = {
                        let mut p = position.clone();
                        p.make_move(&m);
                        p
                    };
                    let mut buf = MoveBuffer::new();
                    -self.negamax(depth - 1, &new_pos, &mut buf)
                })
                .max()
                .unwrap_or_else(||
                    // No moves: checkmate or stalemate
                    if checked {
                        GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, depth))
                    } else {
                        GameStatus::Finished(FinishState::Draw)
                    })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamestate_ordering() {
        /*
        Ordering (from white's perspective)
        * Finished(Some(White))
        * Playing(+)
        * Playing 0 == Finished(None)
        * Playing(-)
        * Finished(Some(Black))
         */

        let combinations = vec![
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Equal,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Equal,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(1.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Equal,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Equal,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(0.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Equal,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Equal,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Draw),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Greater,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Less,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Equal,
            ),
            (
                GameStatus::Playing(Centipawn(-1.0)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Greater,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Me, 3)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Playing(Centipawn(1.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Playing(Centipawn(0.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Finished(FinishState::Draw),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Playing(Centipawn(-1.0)),
                Ordering::Less,
            ),
            (
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                GameStatus::Finished(FinishState::Win(RelativePlayer::Opponent, 3)),
                Ordering::Equal,
            ),
        ];

        for (s, o, expected) in combinations {
            let actual = s.cmp(&o);
            assert_eq!(
                actual, expected,
                "s: {:?}, o: {:?}, expected: {:?}, actual: {:?}",
                s, o, expected, actual
            )
        }
    }
}
