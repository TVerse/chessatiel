use guts::Position;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, one_of, space0, space1};
use nom::combinator::{map, map_res, opt};
use nom::error::{context, VerboseError};
use nom::multi::{count, many0, many1, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::{Finish, IResult};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UciParseError {
    #[error("Error: {0}")]
    Error(String),
}

type Res<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IncomingCommand {
    Uci,
    Debug(bool),
    IsReady,
    UciNewGame,
    Position(Position, Vec<String>),
    Go(GoPayload),
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
            IncomingCommand::Go(payload) => write!(f, "go {}", payload),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct GoPayload {
    pub depth: Option<usize>,
    pub move_time: Option<Duration>,
    pub wtime: Option<Duration>,
    pub winc: Option<Duration>,
    pub btime: Option<Duration>,
    pub binc: Option<Duration>,
}

impl fmt::Display for GoPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(d) = self.depth {
            write!(f, "depth {d} ")?
        };
        if let Some(mt) = self.move_time {
            write!(f, "movetime {} ", mt.as_millis())?
        };
        if let Some(wtime) = self.wtime {
            write!(f, "wtime {} ", wtime.as_millis())?
        };
        if let Some(winc) = self.winc {
            write!(f, "winc {} ", winc.as_millis())?
        };
        if let Some(btime) = self.btime {
            write!(f, "btime {} ", btime.as_millis())?
        };
        if let Some(binc) = self.binc {
            write!(f, "binc {} ", binc.as_millis())?
        };

        Ok(())
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
    context(
        "quit",
        map(alt((tag("quit"), tag("exit"))), |_| IncomingCommand::Quit),
    )(s)
}

fn parse_position(s: &str) -> Res<IncomingCommand> {
    context(
        "position",
        map(
            preceded(
                tuple((tag("position"), space1)),
                tuple((
                    alt((parse_fen, map(tag("startpos"), |_| Position::default()))),
                    opt(preceded(
                        tuple((space0, tag("moves"), space1)),
                        many0(parse_move),
                    )),
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
                Position::from_str(&s)
            },
        ),
    )(s)
}

fn parse_move(s: &str) -> Res<&str> {
    context("move", terminated(alphanumeric1, space0))(s)
}

fn parse_go(s: &str) -> Res<IncomingCommand> {
    context(
        "go",
        map(
            preceded(tag("go"), preceded(space1, parse_go_payload)),
            IncomingCommand::Go,
        ),
    )(s)
}

#[derive(Debug)]
enum GoPayloadOption {
    Depth(usize),
    MoveTime(Duration),
    WTime(Duration),
    BTime(Duration),
    WInc(Duration),
    BInc(Duration),
}

// TODO if times are set they are not independent
fn parse_go_payload(s: &str) -> Res<GoPayload> {
    use GoPayloadOption::*;
    context(
        "go_payload",
        map(
            separated_list1(
                space1,
                alt((
                    map_res(
                        map(separated_pair(tag("depth"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Depth),
                    ),
                    map_res(
                        map(separated_pair(tag("movetime"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Duration::from_millis).map(MoveTime),
                    ),
                    map_res(
                        map(separated_pair(tag("wtime"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Duration::from_millis).map(WTime),
                    ),
                    map_res(
                        map(separated_pair(tag("btime"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Duration::from_millis).map(BTime),
                    ),
                    map_res(
                        map(separated_pair(tag("winc"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Duration::from_millis).map(WInc),
                    ),
                    map_res(
                        map(separated_pair(tag("binc"), space1, digit1), |(_, s)| s),
                        |d: &str| d.parse().map(Duration::from_millis).map(BInc),
                    ),
                )),
            ),
            |gpos| {
                let mut gp = GoPayload::default();
                for gpo in gpos.into_iter() {
                    match gpo {
                        Depth(d) => {
                            gp = GoPayload {
                                depth: Some(d),
                                ..gp
                            }
                        }
                        MoveTime(d) => {
                            gp = GoPayload {
                                move_time: Some(d),
                                ..gp
                            }
                        }
                        WTime(d) => {
                            gp = GoPayload {
                                wtime: Some(d),
                                ..gp
                            }
                        }
                        BTime(d) => {
                            gp = GoPayload {
                                btime: Some(d),
                                ..gp
                            }
                        }
                        WInc(d) => {
                            gp = GoPayload {
                                winc: Some(d),
                                ..gp
                            }
                        }
                        BInc(d) => {
                            gp = GoPayload {
                                binc: Some(d),
                                ..gp
                            }
                        }
                    }
                }
                gp
            },
        ),
    )(s)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutgoingCommand {
    Id(&'static str, &'static str),
    UciOk,
    ReadyOk,
    BestMove(String),
    Info(InfoPayload),
}

impl fmt::Display for OutgoingCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutgoingCommand::Id(k, v) => write!(f, "id {} {}", k, v),
            OutgoingCommand::UciOk => write!(f, "uciok"),
            OutgoingCommand::ReadyOk => write!(f, "readyok"),
            OutgoingCommand::BestMove(m) => write!(f, "bestmove {}", m),
            OutgoingCommand::Info(s) => write!(f, "info {}", s),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InfoPayload {
    String(String),
}

impl fmt::Display for InfoPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfoPayload::String(s) => write!(f, "string {}", s),
        }
    }
}

pub struct UciParser();

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
        .map_err(|e| UciParseError::Error(format!("{:?}", e)))
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
        let input = "position startpos moves e2e4 e7e5";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(
                Position::default(),
                vec!["e2e4".to_owned(), "e7e5".to_owned()],
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
        let input =
            "position rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e5";
        assert_eq!(
            parse_position(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Position(
                Position::default(),
                vec!["e2e4".to_owned(), "e7e5".to_owned()],
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

    #[test]
    fn go_depth() {
        let input = "go depth 4";
        assert_eq!(
            parse_go(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Go(GoPayload {
                depth: Some(4),
                ..GoPayload::default()
            }))
        );
    }

    #[test]
    fn go_movetime() {
        let input = "go movetime 10000";
        assert_eq!(
            parse_go(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Go(GoPayload {
                move_time: Some(Duration::from_millis(10000)),
                ..GoPayload::default()
            }))
        );
    }

    #[test]
    fn go_times() {
        let input = "go wtime 1 winc 2 binc 3 btime 4";
        assert_eq!(
            parse_go(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Go(GoPayload {
                wtime: Some(Duration::from_millis(1)),
                btime: Some(Duration::from_millis(4)),
                winc: Some(Duration::from_millis(2)),
                binc: Some(Duration::from_millis(3)),
                ..GoPayload::default()
            }))
        );
    }

    #[test]
    fn go_times_and_depth() {
        let input = "go wtime 1 winc 2 depth 5 binc 3 btime 4";
        assert_eq!(
            parse_go(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Go(GoPayload {
                wtime: Some(Duration::from_millis(1)),
                btime: Some(Duration::from_millis(4)),
                winc: Some(Duration::from_millis(2)),
                binc: Some(Duration::from_millis(3)),
                depth: Some(5),
                ..GoPayload::default()
            }))
        );
    }

    #[test]
    fn stop() {
        let input = "stop";
        assert_eq!(
            parse_stop(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Stop)
        )
    }

    #[test]
    fn quit() {
        let input = "quit";
        assert_eq!(
            parse_quit(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Quit)
        )
    }

    #[test]
    fn exit_as_quit() {
        let input = "exit";
        assert_eq!(
            parse_quit(input).finish().map(|(_, res)| res),
            Ok(IncomingCommand::Quit)
        )
    }
}
