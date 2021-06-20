mod error;

pub struct UciParser();
use crate::error::UciParseError;
use guts::Position;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, one_of, space0, space1};
use nom::combinator::{map, map_res, opt};
use nom::error::{context, VerboseError};
use nom::multi::{count, many0, many1};
use nom::sequence::{preceded, terminated, tuple};
use nom::{Finish, IResult};
use std::fmt;
use std::str::FromStr;

type Res<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IncomingCommand {
    Uci,
    Debug(bool),
    IsReady,
    SetOption(String, Option<String>),
    UciNewGame,
    Position(Position, Vec<String>),
    Go, // TODO
    Stop,
    Quit,
}

impl fmt::Display for IncomingCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncomingCommand::Uci => write!(f, "uci"),
            IncomingCommand::Debug(d) => write!(f, "debug {}", if *d { "on" } else { "off" }),
            IncomingCommand::IsReady => write!(f, "isready"),
            IncomingCommand::UciNewGame => write!(f, "ucinewgame"),
            IncomingCommand::Position(p, mvs) => write!(
                f,
                "position {}{}",
                p,
                if mvs.is_empty() {
                    "".to_owned()
                } else {
                    format!(" {}", mvs.join(" "))
                }
            ),
            IncomingCommand::Stop => write!(f, "stop"),
            IncomingCommand::Quit => write!(f, "quit"),
            IncomingCommand::SetOption(_, _) => write!(f, "setoption"),
            IncomingCommand::Go => write!(f, "go"),
        }
    }
}

fn parse_uci(s: &str) -> Res<IncomingCommand> {
    context("uci", map(tag("uci"), |_| IncomingCommand::Uci))(s)
}

fn parse_debug(s: &str) -> Res<IncomingCommand> {
    context(
        "debug",
        map(
            preceded(
                tuple((tag("debug"), space1)),
                alt((map(tag("on"), |_| true), map(tag("off"), |_| false))),
            ),
            IncomingCommand::Debug,
        ),
    )(s)
}

fn parse_isready(s: &str) -> Res<IncomingCommand> {
    context("isready", map(tag("isready"), |_| IncomingCommand::IsReady))(s)
}

fn parse_ucinewgame(s: &str) -> Res<IncomingCommand> {
    context(
        "ucinewgame",
        map(tag("ucinewgame"), |_| IncomingCommand::UciNewGame),
    )(s)
}

fn parse_stop(s: &str) -> Res<IncomingCommand> {
    context("stop", map(tag("stop"), |_| IncomingCommand::Stop))(s)
}

fn parse_quit(s: &str) -> Res<IncomingCommand> {
    context("quit", map(tag("quit"), |_| IncomingCommand::Quit))(s)
}

fn parse_position(s: &str) -> Res<IncomingCommand> {
    context(
        "position",
        map(
            preceded(
                tuple((tag("position"), space1)),
                tuple((
                    alt((parse_fen, map(tag("startpos"), |_| Position::default()))),
                    opt(preceded(space0, many0(parse_move))),
                )),
            ),
            |(pos, moves)| {
                IncomingCommand::Position(
                    pos,
                    moves.map_or_else(Vec::new, |v| v.into_iter().map(|s| s.to_owned()).collect()),
                )
            },
        ),
    )(s)
}

fn parse_fen(s: &str) -> Res<Position> {
    context(
        "fen",
        map_res(
            context(
                "fen_count",
                count(
                    context(
                        "fen_terminated",
                        terminated(
                            context(
                                "fen_many1",
                                many1(context("fen_one_of", one_of("/1234567890rnbqkpRNBQKPbw-"))),
                            ),
                            space0,
                        ),
                    ),
                    6,
                ),
            ),
            |v| {
                let s = v
                    .into_iter()
                    .map(|v| v.into_iter().map(|c| c.to_string()).collect_vec().join(""))
                    .collect_vec()
                    .join(" ");
                dbg!(&s);
                Position::from_str(&s)
            },
        ),
    )(s)
}

fn parse_move(s: &str) -> Res<&str> {
    context("move", terminated(alphanumeric1, space0))(s)
}

fn parse_go(s: &str) -> Res<IncomingCommand> {
    context("go", map(tag("go"), |_| IncomingCommand::Go))(s)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutgoingCommand {
    Id(&'static str, &'static str),
    UciOk,
    ReadyOk,
    BestMove(String),
    Info,
    Option,
}

impl fmt::Display for OutgoingCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutgoingCommand::Id(k, v) => write!(f, "id {} {}", k, v),
            OutgoingCommand::UciOk => write!(f, "uciok"),
            OutgoingCommand::ReadyOk => write!(f, "readyok"),
            OutgoingCommand::BestMove(m) => write!(f, "bestmove {}", m),
            OutgoingCommand::Info => write!(f, "info"),
            OutgoingCommand::Option => write!(f, "option"),
        }
    }
}

impl UciParser {
    pub fn new() -> Self {
        Self()
    }

    pub fn parse(&self, s: &str) -> Result<IncomingCommand, UciParseError> {
        alt((
            parse_ucinewgame,
            parse_uci,
            parse_debug,
            parse_isready,
            parse_position,
            parse_stop,
            parse_quit,
            parse_go,
        ))(s)
        .finish()
        .map(|(_, o)| o)
        .map_err(|_| UciParseError::Error("Some error".to_owned()))
    }
}

impl Default for UciParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Finish;

    #[test]
    fn uci() {
        let input = "uci";
        assert_eq!(
            parse_uci(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Uci)
        );
    }

    #[test]
    fn not_uci() {
        let input = "not_uci";
        assert_ne!(
            parse_uci(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Uci)
        );
    }

    #[test]
    fn debug() {
        let input = "debug on";
        assert_eq!(
            parse_debug(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Debug(true))
        );
    }

    #[test]
    fn not_debug() {
        let input = "debug optional";
        assert_ne!(
            parse_debug(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Debug(true))
        );
    }

    #[test]
    fn isready() {
        let input = "isready";
        assert_eq!(
            parse_isready(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::IsReady)
        );
    }

    #[test]
    fn not_isready() {
        let input = "not_isready";
        assert_ne!(
            parse_isready(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::IsReady)
        );
    }

    #[test]
    fn ucinewgame() {
        let input = "ucinewgame";
        assert_eq!(
            parse_ucinewgame(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::UciNewGame)
        );
    }

    #[test]
    fn not_ucinewgame() {
        let input = "not_ucinewgame";
        assert_ne!(
            parse_ucinewgame(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::UciNewGame)
        );
    }

    #[test]
    fn position() {
        let input = "position startpos";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(Position::default(), Vec::new()))
        );
    }

    #[test]
    fn position_with_moves() {
        let input = "position startpos e2e4 e7e5";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(
                Position::default(),
                vec!["e2e4".to_owned(), "e7e5".to_owned()]
            ))
        );
    }

    #[test]
    fn position_with_fen() {
        let input = "position rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(Position::default(), Vec::new()))
        );
    }

    #[test]
    fn position_with_fen_and_moves() {
        let input = "position rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 e2e4 e7e5";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(
                Position::default(),
                vec!["e2e4".to_owned(), "e7e5".to_owned()]
            ))
        );
    }

    #[test]
    fn just_move() {
        let input = "e2e4";
        assert_eq!(parse_move(input).finish().map(|(_, res)| res), Ok("e2e4"));
    }

    #[test]
    fn just_move_promotion() {
        let input = "e2e4q";
        assert_eq!(parse_move(input).finish().map(|(_, res)| res), Ok("e2e4q"));
    }
}