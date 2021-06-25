use crate::bitboard::Bitboard;
use crate::board::Sliders;
use crate::chess_move::MoveType;
use crate::color::Color;
use crate::file::File;
use crate::movegen::movebuffer::MoveBuffer;
use crate::movegen::tables::{KnightMovePatterns, SquaresBetween};
use crate::rank::Rank;
use crate::square::Square;
use crate::{Move, Piece, Position};

pub mod movebuffer;
mod tables;

// TODO Copy/Clone?
#[derive(Debug, Eq, PartialEq)]
struct Pin {
    pinner: Square,
    pinned: Square,
    ray: Bitboard,
}

impl Pin {
    pub fn new(pinner: Square, pinned: Square, ray: Bitboard) -> Self {
        Self {
            pinner,
            pinned,
            ray,
        }
    }
}

// TODO finding stuff in a vec is slow, invert logic?
#[derive(Debug, Eq, PartialEq)]
struct Pins {
    pins: Vec<Pin>,
}

impl Pins {
    pub fn new(pins: Vec<Pin>) -> Self {
        Self { pins }
    }

    pub fn pinned(&self) -> Bitboard {
        Bitboard::from_squares(self.pins.iter().map(|p| p.pinned))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct KingSurroundings {
    checkers: Bitboard,
    pins: Pins,
    king_danger: Bitboard,
}

impl KingSurroundings {
    pub fn new(checkers: Bitboard, pins: Pins, king_danger: Bitboard) -> Self {
        Self {
            checkers,
            pins,
            king_danger,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Masks {
    king_danger: Bitboard,
    capture: Bitboard,
    push: Bitboard,
}

impl Masks {
    pub const fn new(king_danger: Bitboard, capture: Bitboard, push: Bitboard) -> Self {
        Self {
            king_danger,
            capture,
            push,
        }
    }
}

// TODO pull out commonly-used bitboards.
// TODO a lot of these methods don't involve knights and don't need self.
// TODO if the knight table can be const, this struct is obsolete.
pub struct MoveGenerator {
    knight_patterns: KnightMovePatterns,
    squares_between: SquaresBetween,
}

/*
Procedure: (https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/)

1. Find checking pieces.
  * More than one? Only legal moves are king moves.
2. Find king danger squares.
3. Generate capture and push masks. (If not in check, they are the full board). Difference cause en-passant.
4. Find pinned pieces and their legal moves, using masks.
5. Find other pieces' legal moves, using masks.
*/
impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            knight_patterns: KnightMovePatterns::new(),
            squares_between: SquaresBetween::new(),
        }
    }

    pub fn perft(&self, position: &Position, depth: usize) -> usize {
        self.perft_debug(position, depth, false)
    }

    pub fn perft_debug(&self, position: &Position, depth: usize, debug: bool) -> usize {
        if depth == 0 {
            1
        } else {
            let mut buf = MoveBuffer::new();
            let _ = self.generate_legal_moves_for(&position, &mut buf);
            buf.iter().fold(0, |acc, m| {
                let mut position = position.clone();
                position.make_move(m);
                if cfg!(debug_assertions) && debug {
                    println!("{}", m);
                }
                acc + self.perft_debug(&position, depth - 1, false)
            })
        }
    }

    pub fn divide(&self, position: &Position, depth: usize) -> Vec<(Move, usize)> {
        let mut buf = MoveBuffer::new();
        let _ = self.generate_legal_moves_for(&position, &mut buf);
        let mut result = Vec::with_capacity(buf.len());
        for m in buf.iter() {
            let mut position = position.clone();
            position.make_move(m);
            // let debug_flag =
            //     m.from == Square::new(File::D, Rank::R7) && m.to == Square::new(File::D, Rank::R5);
            let debug_flag = false;
            if cfg!(debug_assertions) && debug_flag {
                println!("{}", position);
            }

            let res = self.perft_debug(&position, depth - 1, debug_flag);
            result.push((m.clone(), res));
        }

        result
    }

    // TODO how many moves can a position have in theory? Allocate that much on stack and return a Deref<[Move]>?
    // TODO for statistics and ordering, differentiate between checks/captures/attacks/quiet.
    // TODO currently allows for no friendly king, bench to see if this loses performance.
    // TODO terrible code, refactor
    pub fn generate_legal_moves_for(&self, position: &Position, buffer: &mut MoveBuffer) -> bool {
        buffer.clear();
        let own_pieces = &position.board()[position.active_color()];
        let (KingSurroundings { checkers, pins, .. }, masks) =
            if let Some(own_king_sq) = own_pieces[Piece::King].first_set_square() {
                let ks = self.king_surroundings(position);
                let num_checkers = ks.checkers.count_ones();
                let masks = if num_checkers == 1 {
                    let checker_square = ks.checkers.first_set_square().unwrap(); // Also only set square
                    let piece = position.board().piece_at(checker_square).unwrap();
                    if piece.is_slider() {
                        Masks::new(
                            ks.king_danger,
                            ks.checkers,
                            own_king_sq.ray_between(checker_square),
                        )
                    } else {
                        Masks::new(ks.king_danger, ks.checkers, Bitboard::EMPTY)
                    }
                } else {
                    Masks::new(
                        ks.king_danger,
                        position.board()[!position.active_color()].all_pieces(),
                        !position.board().all_pieces(),
                    )
                };
                (ks, masks)
            } else {
                (
                    KingSurroundings::new(Bitboard::EMPTY, Pins::new(Vec::new()), Bitboard::EMPTY),
                    Masks::new(
                        Bitboard::EMPTY,
                        position.board()[!position.active_color()].all_pieces(),
                        !position.board().all_pieces(),
                    ),
                )
            };

        let num_checkers = checkers.count_ones();
        move_for_king(buffer, position, &masks);

        // Double check (or more), only king moves are possible.
        if num_checkers < 2 {
            self.move_for_pawns(buffer, position, &pins, &masks);

            self.move_for_knights(buffer, position, &pins, &masks);
            self.move_for_cardinals(buffer, position, &pins, &masks);
            self.move_for_diagonals(buffer, position, &pins, &masks);

            if num_checkers == 0 {
                castle(buffer, position, &masks);
            }
        }
        checkers.count_ones() > 0
    }

    fn move_for_knights(
        &self,
        buffer: &mut MoveBuffer,
        position: &Position,
        pins: &Pins,
        masks: &Masks,
    ) {
        let knights = position.board()[position.active_color()][Piece::Knight];
        let knights = knights & !pins.pinned();

        for s in knights.into_iter() {
            let moves = self.knight_patterns.get_move(s);

            buffer.add_capture(Piece::Knight, s, moves & masks.capture);
            buffer.add_push(Piece::Knight, s, moves & masks.push);
        }
    }

    fn move_for_pawns(
        &self,
        buffer: &mut MoveBuffer,
        position: &Position,
        pins: &Pins,
        masks: &Masks,
    ) {
        let own_pieceboard = &position.board()[position.active_color()];
        let own_pawns = own_pieceboard[Piece::Pawn];

        /* TODO
        Can also do this by checking the ray from the pawn forwards.
        Then &forwards for most pawns, &forwards|(forwards.forwards) for home row.
        Makes double moves implicit though and needs more handling in make_move.
         */

        for s in own_pawns {
            let pin_ray = pins
                .pins
                .iter()
                .find(|p| p.pinned == s)
                .map(|p| p.ray)
                .unwrap_or_else(|| Bitboard::FULL);
            let bb = MoveGenerator::pawn_push(buffer, position, masks, s, pin_ray);

            MoveGenerator::pawn_double_push(buffer, position, masks, s, pin_ray, bb);

            MoveGenerator::pawn_captures(buffer, position, masks, s, pin_ray, bb);

            MoveGenerator::pawn_ep(buffer, position, masks, s, pin_ray, bb)
        }
    }

    fn pawn_ep(
        buffer: &mut MoveBuffer,
        position: &Position,
        masks: &Masks,
        s: Square,
        pin_ray: Bitboard,
        bb: Bitboard,
    ) {
        let mut ep = bb.forward_left_one(position.active_color())
            | bb.forward_right_one(position.active_color());
        ep &= position
            .en_passant()
            .map(Bitboard::from_square)
            .unwrap_or(Bitboard::EMPTY);
        ep &= pin_ray;
        if ep != Bitboard::EMPTY {
            // Check for en-passant discovered check
            // Since this is per-pawn, there's only zero or one squares set here.
            // Optimize later.
            for target in ep.into_iter() {
                let target_bb = Bitboard::from_square(target);
                let ep_pawn = target_bb.forward_one(!position.active_color());
                if ((ep_pawn & masks.capture) != Bitboard::EMPTY)
                    || ((bb & masks.push) != Bitboard::EMPTY)
                {
                    let ep_square = ep_pawn.first_set_square().unwrap();
                    let mut all_pieces = position.board().all_pieces();
                    all_pieces &=
                        !Bitboard::from_squares(std::array::IntoIter::new([s, ep_square]));
                    all_pieces |= target_bb;
                    let own_king = position.board()[position.active_color()][Piece::King];
                    let cardinal_attackers = position.board()[!position.active_color()]
                        .sliders()
                        .cardinal;
                    let new_king_attackers =
                        own_king.cardinal_attackers(!all_pieces) & cardinal_attackers;
                    if new_king_attackers != Bitboard::EMPTY {
                        ep &= !target_bb;
                    }
                }
            }
            buffer.add_en_passant(s, ep);
        }
    }

    fn pawn_captures(
        buffer: &mut MoveBuffer,
        position: &Position,
        masks: &Masks,
        s: Square,
        pin_ray: Bitboard,
        bb: Bitboard,
    ) {
        let mut captures = bb.forward_left_one(position.active_color())
            | bb.forward_right_one(position.active_color());
        captures &= masks.capture;
        captures &= pin_ray;
        buffer.add_pawn_capture(s, captures);
    }

    fn pawn_double_push(
        buffer: &mut MoveBuffer,
        position: &Position,
        masks: &Masks,
        s: Square,
        pin_ray: Bitboard,
        bb: Bitboard,
    ) {
        let is_next_square_blocked =
            bb.forward_one(position.active_color()) & position.board().all_pieces();
        if is_next_square_blocked == Bitboard::EMPTY
            && s.rank() == Rank::pawn_two_squares(position.active_color())
        {
            let mut push = bb
                .forward_one(position.active_color())
                .forward_one(position.active_color());
            push &= masks.push;
            push &= pin_ray;
            buffer.add_pawn_push(s, push);
        }
    }

    fn pawn_push(
        buffer: &mut MoveBuffer,
        position: &Position,
        masks: &Masks,
        s: Square,
        pin_ray: Bitboard,
    ) -> Bitboard {
        let bb = Bitboard::from_square(s);
        let mut push = bb.forward_one(position.active_color());
        push &= masks.push;
        push &= pin_ray;
        buffer.add_pawn_push(s, push);
        bb
    }

    fn move_for_cardinals(
        &self,
        buffer: &mut MoveBuffer,
        position: &Position,
        pins: &Pins,
        masks: &Masks,
    ) {
        let own_pieceboard = &position.board()[position.active_color()];
        let own_rooks = own_pieceboard[Piece::Rook];
        let own_queens = own_pieceboard[Piece::Queen];

        for s in own_rooks {
            let pin_ray = pins
                .pins
                .iter()
                .find(|p| p.pinned == s)
                .map(|p| p.ray)
                .unwrap_or_else(|| Bitboard::FULL);
            let bb = Bitboard::from_square(s);
            let mut rays = bb.cardinal_attackers(!position.board().all_pieces());
            rays &= pin_ray;
            rays &= !own_pieceboard.all_pieces();

            buffer.add_push(Piece::Rook, s, rays & masks.push);
            buffer.add_capture(Piece::Rook, s, rays & masks.capture);
        }

        for s in own_queens {
            let pin_ray = pins
                .pins
                .iter()
                .find(|p| p.pinned == s)
                .map(|p| p.ray)
                .unwrap_or_else(|| Bitboard::FULL);
            let bb = Bitboard::from_square(s);
            let mut rays = bb.cardinal_attackers(!position.board().all_pieces());
            rays &= pin_ray;
            rays &= !own_pieceboard.all_pieces();

            buffer.add_push(Piece::Queen, s, rays & masks.push);
            buffer.add_capture(Piece::Queen, s, rays & masks.capture);
        }
    }

    fn move_for_diagonals(
        &self,
        buffer: &mut MoveBuffer,
        position: &Position,
        pins: &Pins,
        masks: &Masks,
    ) {
        let own_pieceboard = &position.board()[position.active_color()];
        let own_bishops = own_pieceboard[Piece::Bishop];
        let own_queens = own_pieceboard[Piece::Queen];

        for s in own_bishops {
            let pin_ray = pins
                .pins
                .iter()
                .find(|p| p.pinned == s)
                .map(|p| p.ray)
                .unwrap_or_else(|| Bitboard::FULL);
            let bb = Bitboard::from_square(s);
            let mut rays = bb.diagonal_attackers(!position.board().all_pieces());
            rays &= pin_ray;
            rays &= !own_pieceboard.all_pieces();

            buffer.add_push(Piece::Bishop, s, rays & masks.push);
            buffer.add_capture(Piece::Bishop, s, rays & masks.capture);
        }

        for s in own_queens {
            let pin_ray = pins
                .pins
                .iter()
                .find(|p| p.pinned == s)
                .map(|p| p.ray)
                .unwrap_or_else(|| Bitboard::FULL);
            let bb = Bitboard::from_square(s);
            let mut rays = bb.diagonal_attackers(!position.board().all_pieces());
            rays &= pin_ray;
            rays &= !own_pieceboard.all_pieces();

            buffer.add_push(Piece::Queen, s, rays & masks.push);
            buffer.add_capture(Piece::Queen, s, rays & masks.capture);
        }
    }

    fn king_surroundings(&self, position: &Position) -> KingSurroundings {
        let own_pieceboard = &position.board()[position.active_color()];
        let own_pieces = own_pieceboard.all_pieces();
        let own_king = own_pieceboard[Piece::King];
        let own_king_sq = own_king.first_set_square().unwrap();
        let opponent = !position.active_color();
        let enemy_pieceboard = &position.board()[opponent];
        let occupied = position.board().all_pieces();

        // Pawns and knights cannot pin
        let knight_check =
            self.knight_patterns.get_moves(own_king) & enemy_pieceboard[Piece::Knight];
        let enemy_pawns = enemy_pieceboard[Piece::Pawn];
        let pawn_check = (own_king.forward_left_one(position.active_color())
            | own_king.forward_right_one(position.active_color()))
            & enemy_pawns;

        let enemy_cardinal = enemy_pieceboard[Piece::Rook] | enemy_pieceboard[Piece::Queen];
        let enemy_diagonal = enemy_pieceboard[Piece::Bishop] | enemy_pieceboard[Piece::Queen];

        // These two can probably be lookup tables?
        let cardinal_attackers = own_king.cardinal_attackers(Bitboard::FULL) & enemy_cardinal;
        let diagonal_attackers = own_king.diagonal_attackers(Bitboard::FULL) & enemy_diagonal;

        let attackers = cardinal_attackers | diagonal_attackers;

        // TODO vec size
        // TODO Use array instead to save the heap allocation, max amount of pinners is 8.
        // TODO I think these need to be paired so bitboards are not enough?
        let mut pins = Vec::with_capacity(8);
        let mut checkers = Bitboard::EMPTY;
        attackers.into_iter().for_each(|s| {
            let attacker = Bitboard::from_square(s);
            let ray = self.squares_between.between(s, own_king_sq);
            let pieces_between = ray & occupied;

            if pieces_between == Bitboard::EMPTY {
                // Nothing between? Check.
                checkers |= attacker
            } else if pieces_between.count_ones() == 1
                && (pieces_between & own_pieces).count_ones() == 1
            {
                // One piece between and it's ours? Pinned.
                let pinner = s;
                let pinned = pieces_between.first_set_square().unwrap();
                pins.push(Pin::new(pinner, pinned, ray | attacker));
            } else {
                // Nothing
            }
        });

        let checkers = checkers | pawn_check | knight_check;

        KingSurroundings::new(checkers, Pins::new(pins), self.king_danger(position))
    }

    fn king_danger(&self, position: &Position) -> Bitboard {
        let opponent = !position.active_color();

        let Sliders { cardinal, diagonal } = position.board().sliders(opponent);
        let all_except_king =
            position.board().all_pieces() & !position.board()[position.active_color()][Piece::King];
        let empty = !all_except_king;

        let cardinal = cardinal.cardinal_attackers(empty);
        let diagonal = diagonal.diagonal_attackers(empty);

        let sliders = cardinal | diagonal;

        let opponent_pieces = &position.board()[opponent];
        let knights = opponent_pieces[Piece::Knight];
        let knights = self.knight_patterns.get_moves(knights);

        let pawns = opponent_pieces[Piece::Pawn];
        let pawns = pawns.forward_left_one(opponent) | pawns.forward_right_one(opponent);

        let kings = opponent_pieces[Piece::King];
        let kings = kings.surrounding();

        sliders | knights | pawns | kings
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn castle(buffer: &mut MoveBuffer, position: &Position, masks: &Masks) {
    let king_from = match position.active_color() {
        Color::White => Square::new(File::E, Rank::R1),
        Color::Black => Square::new(File::E, Rank::R8),
    };

    if position.castle_rights()[position.active_color()].kingside {
        debug_assert_eq!(
            position.board()[position.active_color()].piece_at(king_from),
            Some(Piece::King),
            "Expected king, got {:?} from position {}",
            position.board()[position.active_color()].piece_at(king_from),
            &position
        );
        let king_target = match position.active_color() {
            Color::White => Square::new(File::G, Rank::R1),
            Color::Black => Square::new(File::G, Rank::R8),
        };

        let rook_pos = match position.active_color() {
            Color::White => Square::new(File::H, Rank::R1),
            Color::Black => Square::new(File::H, Rank::R8),
        };

        debug_assert_eq!(
            position.board()[position.active_color()].piece_at(rook_pos),
            Some(Piece::Rook),
            "Expected rook, got {:?} from position {}",
            position.board()[position.active_color()].piece_at(king_from),
            &position
        );

        let king_move_squares = match position.active_color() {
            Color::White => Bitboard::from_squares(
                [
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Square::new(File::G, Rank::R1),
                ]
                .iter()
                .copied(),
            ),
            Color::Black => Bitboard::from_squares(
                [
                    Square::new(File::E, Rank::R8),
                    Square::new(File::F, Rank::R8),
                    Square::new(File::G, Rank::R8),
                ]
                .iter()
                .copied(),
            ),
        };

        if (king_move_squares & masks.king_danger)
            | (rook_pos.ray_between(king_from) & position.board().all_pieces())
            == Bitboard::EMPTY
        {
            buffer.add_castle(king_from, king_target, MoveType::CASTLE_KINGISDE)
        }
    }

    if position.castle_rights()[position.active_color()].queenside {
        debug_assert_eq!(
            position.board()[position.active_color()].piece_at(king_from),
            Some(Piece::King),
            "Expected king, got {:?} from position {}",
            position.board()[position.active_color()].piece_at(king_from),
            &position
        );
        let king_target = match position.active_color() {
            Color::White => Square::new(File::C, Rank::R1),
            Color::Black => Square::new(File::C, Rank::R8),
        };

        let rook_pos = match position.active_color() {
            Color::White => Square::new(File::A, Rank::R1),
            Color::Black => Square::new(File::A, Rank::R8),
        };

        debug_assert_eq!(
            position.board()[position.active_color()].piece_at(rook_pos),
            Some(Piece::Rook),
            "Expected rook, got {:?} from position {}",
            position.board()[position.active_color()].piece_at(king_from),
            &position
        );

        let king_move_squares = match position.active_color() {
            Color::White => Bitboard::from_squares(
                [
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Square::new(File::C, Rank::R1),
                ]
                .iter()
                .copied(),
            ),
            Color::Black => Bitboard::from_squares(
                [
                    Square::new(File::E, Rank::R8),
                    Square::new(File::D, Rank::R8),
                    Square::new(File::C, Rank::R8),
                ]
                .iter()
                .copied(),
            ),
        };

        if (king_move_squares & masks.king_danger)
            | (rook_pos.ray_between(king_from) & position.board().all_pieces())
            == Bitboard::EMPTY
        {
            buffer.add_castle(king_from, king_target, MoveType::CASTLE_QUEENSIDE)
        }
    }
}

fn move_for_king(buffer: &mut MoveBuffer, position: &Position, masks: &Masks) {
    let own_pieces = &position.board()[position.active_color()];
    let king = own_pieces[Piece::King];
    if let Some(king_square) = king.first_set_square() {
        let candidate_squares = king.surrounding();
        let possible_squares = (candidate_squares & !masks.king_danger) & !own_pieces.all_pieces();

        buffer.add_push(
            Piece::King,
            king_square,
            possible_squares & !position.board()[!position.active_color()].all_pieces(),
        );
        buffer.add_capture(
            Piece::King,
            king_square,
            possible_squares & position.board()[!position.active_color()].all_pieces(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::{EitherOrBoth, Itertools};

    use crate::file::File;
    use crate::rank::Rank;
    use crate::square::Square;

    use super::*;

    fn pretty_error(moves: &[Move], expected: &[Move]) -> String {
        moves
            .iter()
            .zip_longest(expected.iter())
            .fold(String::new(), |acc, eob| {
                format!(
                    "{}{}\n",
                    acc,
                    match eob {
                        EitherOrBoth::Both(m, e) => format!("{} {}", m, e),
                        EitherOrBoth::Left(m) => format!("{} MISS", m),
                        EitherOrBoth::Right(e) => format!("MISS {}", e),
                    }
                )
            })
    }

    fn compare_moves<F>(starting_position_fen: &'static str, filter: F, expected: &mut [Move])
    where
        F: FnMut(&&Move) -> bool,
    {
        let generator = MoveGenerator::new();

        let starting_position = Position::from_str(starting_position_fen).unwrap();

        let mut buf = MoveBuffer::new();
        let _checked = generator.generate_legal_moves_for(&starting_position, &mut buf);
        let mut moves: Vec<_> = buf.iter().filter(filter).map(|m| m.clone()).collect();

        moves.sort();

        expected.sort();

        let pretty_error = pretty_error(moves.as_slice(), expected);

        assert_eq!(moves, expected, "\n{}", pretty_error)
    }

    #[test]
    fn test_king_danger() {
        let generator = MoveGenerator::new();

        let position = Position::from_str("r7/8/8/8/8/8/1K6/8 w - - 0 1").unwrap();

        let expected =
            (Bitboard::A_FILE | Bitboard::RANK_8) & !(Bitboard::A_FILE & Bitboard::RANK_8);

        assert_eq!(generator.king_danger(&position), expected)
    }

    #[test]
    fn test_king_surroundings() {
        let generator = MoveGenerator::new();

        let position = Position::from_str("1r6/8/5q2/4P3/8/8/1K1B1r2/b4r2 w - - 0 1").unwrap();
        let surroundings = generator.king_surroundings(&position);

        let expected_checkers = Bitboard::from_squares(
            vec![
                Square::new(File::B, Rank::R8),
                Square::new(File::A, Rank::R1),
            ]
            .into_iter(),
        );

        let expected_pins = Pins::new(vec![
            Pin {
                pinner: Square::new(File::F, Rank::R2),
                pinned: Square::new(File::D, Rank::R2),
                ray: Bitboard::from_squares(
                    vec![
                        Square::new(File::C, Rank::R2),
                        Square::new(File::D, Rank::R2),
                        Square::new(File::E, Rank::R2),
                        Square::new(File::F, Rank::R2),
                    ]
                    .into_iter(),
                ),
            },
            Pin {
                pinner: Square::new(File::F, Rank::R6),
                pinned: Square::new(File::E, Rank::R5),
                ray: Bitboard::from_squares(
                    vec![
                        Square::new(File::C, Rank::R3),
                        Square::new(File::D, Rank::R4),
                        Square::new(File::E, Rank::R5),
                        Square::new(File::F, Rank::R6),
                    ]
                    .into_iter(),
                ),
            },
        ]);

        assert_eq!(surroundings.checkers, expected_checkers);
        assert_eq!(surroundings.pins, expected_pins);
    }

    #[test]
    fn king_in_corner() {
        compare_moves(
            "8/8/8/8/8/8/8/K7 w - - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::A, Rank::R1),
                    Square::new(File::A, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::A, Rank::R1),
                    Square::new(File::B, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::A, Rank::R1),
                    Square::new(File::B, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn king_in_corner_cut_off() {
        compare_moves(
            "1r6/8/8/8/8/8/8/K7 w - - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![Move::new(
                Square::new(File::A, Rank::R1),
                Square::new(File::A, Rank::R2),
                Piece::King,
                MoveType::PUSH,
                None,
            )],
        )
    }

    #[test]
    fn king_boxed_in() {
        compare_moves(
            "1r6/8/8/8/8/8/PN6/KN6 w - - 0 1",
            |m| m.piece == Piece::King,
            &mut Vec::new(),
        )
    }

    #[test]
    fn pinned_knight_no_moves() {
        compare_moves(
            "1r6/8/8/8/8/8/1N6/1K6 w - - 0 1",
            |m| m.piece == Piece::Knight,
            &mut Vec::new(),
        )
    }

    #[test]
    fn knight() {
        compare_moves(
            "8/8/2p5/8/3N4/8/2P5/8 w - - 0 1",
            |m| m.piece == Piece::Knight,
            &mut vec![
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::B, Rank::R3),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::B, Rank::R5),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::C, Rank::R6),
                    Piece::Knight,
                    MoveType::CAPTURE,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::E, Rank::R6),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::F, Rank::R5),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::F, Rank::R3),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::E, Rank::R2),
                    Piece::Knight,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn pawns() {
        compare_moves(
            "8/8/8/2r1rpP1/3P3r/1P3b2/P5PP/7K w - f6 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![
                Move::new(
                    Square::new(File::A, Rank::R2),
                    Square::new(File::A, Rank::R3),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::A, Rank::R2),
                    Square::new(File::A, Rank::R4),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::B, Rank::R3),
                    Square::new(File::B, Rank::R4),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::C, Rank::R5),
                    Piece::Pawn,
                    MoveType::CAPTURE,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::E, Rank::R5),
                    Piece::Pawn,
                    MoveType::CAPTURE,
                    None,
                ),
                Move::new(
                    Square::new(File::D, Rank::R4),
                    Square::new(File::D, Rank::R5),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::G, Rank::R5),
                    Square::new(File::G, Rank::R6),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::G, Rank::R5),
                    Square::new(File::F, Rank::R6),
                    Piece::Pawn,
                    MoveType::CAPTURE | MoveType::EN_PASSANT,
                    None,
                ),
                Move::new(
                    Square::new(File::G, Rank::R2),
                    Square::new(File::F, Rank::R3),
                    Piece::Pawn,
                    MoveType::CAPTURE,
                    None,
                ),
                Move::new(
                    Square::new(File::H, Rank::R2),
                    Square::new(File::H, Rank::R3),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn pawns_cannot_jump() {
        compare_moves(
            "8/8/8/8/8/N7/P7/8 w - - 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![],
        )
    }

    #[test]
    fn perft_position_1() {
        compare_moves(
            "rnbqkbnr/ppp1pppp/8/3p4/8/1P6/P1PPPPPP/RNBQKBNR w KQkq - 0 1",
            |m| m.piece == Piece::Bishop,
            &mut vec![
                Move::new(
                    Square::new(File::C, Rank::R1),
                    Square::new(File::B, Rank::R2),
                    Piece::Bishop,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::C, Rank::R1),
                    Square::new(File::A, Rank::R3),
                    Piece::Bishop,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn king_move_out_of_check() {
        compare_moves(
            "8/8/8/8/4r3/8/8/4K3 w - - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn castle() {
        compare_moves(
            "8/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::E, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::C, Rank::R1),
                    Piece::King,
                    MoveType::CASTLE_QUEENSIDE,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::G, Rank::R1),
                    Piece::King,
                    MoveType::CASTLE_KINGISDE,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn castle_no_rights() {
        compare_moves(
            "8/8/8/8/8/8/8/R3K2R w - - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::E, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn no_castle_in_check() {
        compare_moves(
            "8/8/8/8/4r3/8/8/R3K2R w KQ - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn no_castle_through_check() {
        compare_moves(
            "8/8/8/8/3r1r2/8/8/R3K2R w KQ - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![Move::new(
                Square::new(File::E, Rank::R1),
                Square::new(File::E, Rank::R2),
                Piece::King,
                MoveType::PUSH,
                None,
            )],
        )
    }

    #[test]
    fn no_castle_through_pieces() {
        compare_moves(
            "8/8/8/8/8/8/8/Rb2K1NR w KQ - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R1),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::D, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::E, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R1),
                    Square::new(File::F, Rank::R2),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn promotion() {
        compare_moves(
            "8/4P3/8/8/8/8/8/8 w - - 0 1",
            |_| true,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::E, Rank::R8),
                    Piece::Pawn,
                    MoveType::PUSH,
                    Some(Piece::Knight),
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::E, Rank::R8),
                    Piece::Pawn,
                    MoveType::PUSH,
                    Some(Piece::Bishop),
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::E, Rank::R8),
                    Piece::Pawn,
                    MoveType::PUSH,
                    Some(Piece::Rook),
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::E, Rank::R8),
                    Piece::Pawn,
                    MoveType::PUSH,
                    Some(Piece::Queen),
                ),
            ],
        )
    }

    #[test]
    fn king_away_from_checking_slider() {
        compare_moves(
            "8/4k3/8/8/4R3/8/8/4K3 b - - 0 1",
            |m| m.piece == Piece::King,
            &mut vec![
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::D, Rank::R8),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::D, Rank::R7),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::D, Rank::R6),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::F, Rank::R8),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::F, Rank::R7),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::E, Rank::R7),
                    Square::new(File::F, Rank::R6),
                    Piece::King,
                    MoveType::PUSH,
                    None,
                ),
            ],
        )
    }

    #[test]
    fn en_passant_check_evasion_capture() {
        compare_moves(
            "8/8/8/2k5/3Pp3/8/8/K7 b - d3 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![Move::new(
                Square::new(File::E, Rank::R4),
                Square::new(File::D, Rank::R3),
                Piece::Pawn,
                MoveType::CAPTURE | MoveType::EN_PASSANT,
                None,
            )],
        )
    }

    #[test]
    fn en_passant_check_evasion_push() {
        compare_moves(
            "8/8/8/1k6/3Pp3/8/4Q3/K7 b - d3 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![Move::new(
                Square::new(File::E, Rank::R4),
                Square::new(File::D, Rank::R3),
                Piece::Pawn,
                MoveType::CAPTURE | MoveType::EN_PASSANT,
                None,
            )],
        )
    }

    #[test]
    fn en_passant_discovered_check() {
        compare_moves(
            "8/8/8/8/1k1Pp2Q/8/8/K7 b - d3 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![Move::new(
                Square::new(File::E, Rank::R4),
                Square::new(File::E, Rank::R3),
                Piece::Pawn,
                MoveType::PUSH,
                None,
            )],
        )
    }

    #[test]
    fn king_capture_non_checking_piece() {
        compare_moves(
            "8/8/8/8/6b1/8/2Pn4/2RKB3 w - - 0 1",
            |_m| true,
            &mut vec![Move::new(
                Square::new(File::D, Rank::R1),
                Square::new(File::D, Rank::R2),
                Piece::King,
                MoveType::CAPTURE,
                None,
            )],
        )
    }

    #[test]
    fn en_passant_is_not_discovered_check() {
        compare_moves(
            "k2q4/8/8/2Pp4/8/8/8/3K4 w - d6 0 1",
            |m| m.piece == Piece::Pawn,
            &mut vec![
                Move::new(
                    Square::new(File::C, Rank::R5),
                    Square::new(File::C, Rank::R6),
                    Piece::Pawn,
                    MoveType::PUSH,
                    None,
                ),
                Move::new(
                    Square::new(File::C, Rank::R5),
                    Square::new(File::D, Rank::R6),
                    Piece::Pawn,
                    MoveType::CAPTURE | MoveType::EN_PASSANT,
                    None,
                ),
            ],
        )
    }
}
