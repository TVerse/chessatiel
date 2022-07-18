use crate::brain::SHARED_COMPONENTS;
use crate::AnswerTx;
use crate::{ack, answer, AckTx};
use guts::Move;
use guts::{MoveBuffer, MoveGenerator, Position};
use tokio::sync::mpsc;

enum PositionHistoryMessage {
    ResetPosition(AckTx, Position),
    SetMoves(AckTx, Vec<Move>),
    SetMoveStrings(AckTx, Vec<String>),
    Push(AckTx, Position),
    Pop(AnswerTx<Position>),
    IsThreefoldRepetition(AnswerTx<bool>),
    GetCurrentPosition(AnswerTx<Position>),
}

#[derive(Clone)]
pub struct PositionHistoryHandle {
    sender: mpsc::UnboundedSender<PositionHistoryMessage>,
}

impl PositionHistoryHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut actor = PositionHistoryActor::new(receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn reset_position(&self, position: Position) {
        let (tx, rx) = ack();
        let msg = PositionHistoryMessage::ResetPosition(tx, position);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn set_moves(&self, moves: Vec<Move>) {
        let (tx, rx) = ack();
        let msg = PositionHistoryMessage::SetMoves(tx, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn set_move_strings(&self, moves: Vec<String>) {
        let (tx, rx) = ack();
        let msg = PositionHistoryMessage::SetMoveStrings(tx, moves);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn push(&self, position: Position) {
        let (tx, rx) = ack();
        let msg = PositionHistoryMessage::Push(tx, position);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn pop(&self) -> Position {
        let (tx, rx) = answer();
        let msg = PositionHistoryMessage::Pop(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn is_threefold_repetition(&self) -> bool {
        let (tx, rx) = answer();
        let msg = PositionHistoryMessage::IsThreefoldRepetition(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }

    pub async fn get_current_position(&self) -> Position {
        let (tx, rx) = answer();
        let msg = PositionHistoryMessage::GetCurrentPosition(tx);

        let _ = self.sender.send(msg);
        rx.await.expect("Actor task was killed")
    }
}

#[derive(Debug)]
struct PositionHistoryActor {
    receiver: mpsc::UnboundedReceiver<PositionHistoryMessage>,
    positions: Vec<Position>,
}

impl PositionHistoryActor {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_event(msg).await;
        }
    }

    async fn handle_event(&mut self, message: PositionHistoryMessage) {
        match message {
            PositionHistoryMessage::ResetPosition(ack, pos) => {
                self.positions = vec![pos];
                let _ = ack.send(());
            }
            PositionHistoryMessage::SetMoves(ack, moves) => {
                self.set_moves(&moves);
                let _ = ack.send(());
            }
            PositionHistoryMessage::SetMoveStrings(ack, strings) => {
                self.set_moves_from_strings(&strings, &SHARED_COMPONENTS.move_generator);
                let _ = ack.send(());
            }
            PositionHistoryMessage::Push(ack, pos) => {
                self.push(pos);
                let _ = ack.send(());
            }
            PositionHistoryMessage::Pop(answer) => {
                answer.send(self.pop()).unwrap();
            }
            PositionHistoryMessage::IsThreefoldRepetition(answer) => {
                answer.send(self.is_threefold_repetition()).unwrap();
            }
            PositionHistoryMessage::GetCurrentPosition(answer) => {
                answer.send(self.current_position().clone()).unwrap();
            }
        }
    }

    fn new(receiver: mpsc::UnboundedReceiver<PositionHistoryMessage>) -> Self {
        Self {
            receiver,
            positions: vec![Position::default()],
        }
    }

    fn set_moves(&mut self, moves: &[Move]) {
        let mut positions = vec![self.positions[0].clone()];
        positions.extend(moves.iter().scan(self.positions[0].clone(), |p, m| {
            p.make_move(m);

            Some(p.clone())
        }));

        self.positions = positions;
    }

    fn set_moves_from_strings(&mut self, moves: &[String], move_generator: &MoveGenerator) {
        let mut positions = vec![self.positions[0].clone()];
        let mut buf = MoveBuffer::new();
        positions.extend(moves.iter().scan(self.positions[0].clone(), |p, m| {
            let _ = move_generator.generate_legal_moves_for(p, &mut buf);

            let found_move = buf
                .moves
                .iter()
                .find(|fm| &fm.as_uci() == m)
                .unwrap_or_else(|| panic!("Got invalid move {m}"));

            p.make_move(found_move);

            Some(p.clone())
        }));

        self.positions = positions;
    }

    fn current_position(&self) -> &Position {
        self.positions.last().unwrap()
    }

    fn push(&mut self, position: Position) {
        self.positions.push(position)
    }

    fn pop(&mut self) -> Position {
        // TODO should never be empty during search
        self.positions.pop().unwrap()
    }

    fn is_threefold_repetition(&self) -> bool {
        self.positions.iter().fold(0, |count, p| {
            if p.repetition_compare(self.current_position()) {
                count + 1
            } else {
                count
            }
        }) >= 3
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use guts::{File, MoveType, Piece, Rank, Square};
//     use std::str::FromStr;
//
//     #[test]
//     fn test_position_history() {
//         let mut ph =
//             PositionHistoryActor::new(Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap());
//         let moves = [
//             Move::new(
//                 Square::new(File::B, Rank::R2),
//                 Square::new(File::B, Rank::R3),
//                 Piece::King,
//                 MoveType::PUSH,
//                 None,
//             ),
//             Move::new(
//                 Square::new(File::D, Rank::R7),
//                 Square::new(File::E, Rank::R6),
//                 Piece::King,
//                 MoveType::PUSH,
//                 None,
//             ),
//         ];
//         ph.set_moves(&moves);
//         let expected = PositionHistoryActor {
//             positions: vec![
//                 Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
//                 Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap(),
//                 Position::from_str("8/8/4k3/8/8/1K6/8/8 w - - 2 2").unwrap(),
//             ],
//         };
//
//         assert_eq!(ph, expected);
//         let moves = [Move::new(
//             Square::new(File::B, Rank::R2),
//             Square::new(File::B, Rank::R3),
//             Piece::King,
//             MoveType::PUSH,
//             None,
//         )];
//         ph.set_moves(&moves);
//         let expected = PositionHistoryActor {
//             positions: vec![
//                 Position::from_str("8/3k4/8/8/8/8/1K6/8 w - - 0 1").unwrap(),
//                 Position::from_str("8/3k4/8/8/8/1K6/8/8 b - - 1 1").unwrap(),
//             ],
//         };
//         assert_eq!(ph, expected);
//     }
//
//     #[test]
//     fn test_threefold() {
//         let mut ph = PositionHistoryActor::new(Position::default());
//         ph.push(Position::default());
//         ph.push(Position::default());
//
//         assert!(ph.is_threefold_repetition());
//     }
//
//     #[test]
//     fn test_not_threefold() {
//         let mut ph = PositionHistoryActor::new(Position::default());
//         ph.push(Position::default());
//         ph.push(Position::from_str("k7/8/4r3/8/8/4R3/8/K7 w - - 0 1").unwrap());
//
//         assert!(!ph.is_threefold_repetition());
//     }
// }
