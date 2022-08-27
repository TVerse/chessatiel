use crate::{AnnotatedPosition, GameResult};
use anyhow::{anyhow, Result};
use guts::{File, MoveBuffer, MoveGenerator, MoveType, Piece, Position, Rank, Square};
use itertools::Itertools;
use std::str::FromStr;

use rayon::prelude::*;

#[derive(Debug)]
pub enum MoveParseResult {
    Standard {
        piece: Piece,
        target: Square,
        promotion: Option<Piece>,
        from_file: Option<File>,
        from_rank: Option<Rank>,
    },
    KingsideCastle,
    QueensideCastle,
}

pub fn pgn_to_annotated_fen(raw: &str) -> Result<Vec<AnnotatedPosition>> {
    Ok(raw
        .split("[Event")
        .collect_vec()
        .into_par_iter()
        .filter(|f| !f.is_empty())
        .map(|s| {
            s.lines()
                .filter(|l| !l.starts_with('[') && !l.is_empty() && !l.ends_with(']'))
                .map(|s| s.trim())
                .join(" ")
        })
        .map(|g| parse_moves(&g))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|(ps, result)| {
            ps.into_iter()
                .map(move |pos| AnnotatedPosition { pos, result })
        })
        .collect())
}

fn parse_moves(list: &str) -> Result<(Vec<Position>, GameResult)> {
    let movegen = MoveGenerator::new();

    let split = list
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .filter(|s| !s.ends_with('.'))
        .filter(|&s| s != "1-0" && s != "0-1" && s != "1/2-1/2")
        .map(str::trim);

    let gameresult = list
        .split_whitespace()
        .find(|&s| s == "1-0" || s == "0-1" || s == "1/2-1/2")
        .ok_or_else(|| anyhow!("Did not find game result in {list}"))?;
    let gameresult = match gameresult {
        "1-0" => GameResult::White,
        "0-1" => GameResult::Black,
        "1/2-1/2" => GameResult::Draw,
        _ => unreachable!("Found a strange gameresult {gameresult} in {list}"),
    };

    let mut cur_pos = Position::default();

    let mut res = Vec::new();

    let mut buf = MoveBuffer::new();
    for s in split {
        let mpr = parse_move(s)?;
        let _ = movegen.generate_legal_moves_for(&cur_pos, &mut buf);
        let m = buf
            .iter()
            .find(|&m| match mpr {
                MoveParseResult::Standard {
                    piece,
                    target,
                    promotion,
                    from_file,
                    from_rank,
                } => {
                    let known =
                        piece == m.piece() && target == m.to() && promotion == m.promotion();

                    let file = if let Some(f) = from_file {
                        f == m.from().file()
                    } else {
                        true
                    };

                    let rank = if let Some(r) = from_rank {
                        r == m.from().rank()
                    } else {
                        true
                    };

                    known && file && rank
                }
                MoveParseResult::KingsideCastle => {
                    m.move_type().contains(MoveType::CASTLE_KINGSIDE)
                }
                MoveParseResult::QueensideCastle => {
                    m.move_type().contains(MoveType::CASTLE_QUEENSIDE)
                }
            })
            .ok_or_else(|| {
                anyhow!(
                    "Didn't find move in position {} matching {:?} in list '{list}'",
                    cur_pos,
                    mpr
                )
            })?;

        res.push(cur_pos.clone());
        cur_pos.make_move(m);
    }

    Ok((res, gameresult))
}

fn parse_move(m: &str) -> Result<MoveParseResult> {
    if m.starts_with("O-O-O") {
        return Ok(MoveParseResult::QueensideCastle);
    }
    if m.starts_with("O-O") {
        return Ok(MoveParseResult::KingsideCastle);
    }

    let promotion = m
        .find('=')
        .and_then(|idx| m.chars().nth(idx + 1))
        .map(parse_piece)
        .transpose()?;
    let m_no_x = m.chars().filter(|&c| c != 'x').collect::<String>();
    let piece = if m_no_x.starts_with(|c: char| c.is_lowercase()) {
        Piece::Pawn
    } else {
        parse_piece(m_no_x.chars().next().unwrap())?
    };
    let m_no_piece_no_promotion_no_checks = {
        let m_no_promotion = m_no_x.split('=').next().unwrap();
        if m_no_promotion.starts_with(|c: char| c.is_lowercase()) {
            m_no_promotion.to_string()
        } else {
            m_no_promotion.chars().dropping(1).collect::<String>()
        }
        .trim_matches('+')
        .trim_matches('#')
        .to_string()
    };

    let target_str = m_no_piece_no_promotion_no_checks
        .chars()
        .rev()
        .take(2)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let target = Square::from_str(&target_str)
        .map_err(|_| anyhow!("Couldn't find square from {target_str}"))?;
    let (from_file, from_rank) = if m_no_piece_no_promotion_no_checks.len() > 2 {
        if m_no_piece_no_promotion_no_checks.len() == 4 {
            (
                Some(
                    File::try_from(m_no_piece_no_promotion_no_checks.chars().next().unwrap())
                        .unwrap(),
                ),
                Some(
                    Rank::try_from(m_no_piece_no_promotion_no_checks.chars().nth(1).unwrap())
                        .unwrap(),
                ),
            )
        } else if m_no_piece_no_promotion_no_checks.len() == 3 {
            let first = m_no_piece_no_promotion_no_checks.chars().next().unwrap();
            let file = File::try_from(first).ok();
            let rank = Rank::try_from(first).ok();
            (file, rank)
        } else {
            panic!()
        }
    } else {
        (None, None)
    };

    Ok(MoveParseResult::Standard {
        piece,
        target,
        promotion,
        from_file,
        from_rank,
    })
}

fn parse_piece(m: char) -> Result<Piece> {
    match m {
        'Q' => Ok(Piece::Queen),
        'R' => Ok(Piece::Rook),
        'B' => Ok(Piece::Bishop),
        'N' => Ok(Piece::Knight),
        'K' => Ok(Piece::King),
        _ => Err(anyhow!("Unknown piece short {m}")),
    }
}
