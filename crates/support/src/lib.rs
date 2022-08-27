use guts::Position;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub mod pgn;
pub mod pst_optimization;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameResult {
    White,
    Black,
    Draw,
}

impl From<GameResult> for f32 {
    fn from(gr: GameResult) -> Self {
        match gr {
            GameResult::White => 1.0,
            GameResult::Black => -1.0,
            GameResult::Draw => 0.0,
        }
    }
}

impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameResult::White => "1-0",
                GameResult::Black => "0-1",
                GameResult::Draw => "1/2-1/2",
            }
        )
    }
}

impl FromStr for GameResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1-0" => Ok(GameResult::White),
            "0-1" => Ok(GameResult::Black),
            "1/2-1/2" => Ok(GameResult::Draw),
            _ => Err(format!("Unknown gameresult {s}")),
        }
    }
}

pub struct AnnotatedPosition {
    pos: Position,
    result: GameResult,
}

impl Display for AnnotatedPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.pos, self.result)
    }
}

impl FromStr for AnnotatedPosition {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let last = value
            .split_whitespace()
            .last()
            .ok_or_else(|| "Got empty string when parsing annotated position".to_owned())?;
        let result = GameResult::from_str(last)?;
        let rest = value.split_whitespace().take(6).join(" ");
        let pos =
            Position::from_str(&rest).map_err(|e| format!("Fen parse error on {value}: {e}"))?;
        Ok(Self { pos, result })
    }
}
