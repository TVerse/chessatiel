use beak::{IncomingCommand, OutgoingCommand};
use brain::Engine;
use guts::Position;
use log::*;
use std::sync::mpsc::{Receiver, Sender};

pub struct EngineManager {
    engine: Option<Engine>,
    cur_pos: Position,
    rx: Receiver<IncomingCommand>,
    tx: Sender<OutgoingCommand>,
}
impl EngineManager {
    pub fn new(stdin_rx: Receiver<IncomingCommand>, stdout_tx: Sender<OutgoingCommand>) -> Self {
        Self {
            engine: None,
            cur_pos: Position::default(),
            rx: stdin_rx,
            tx: stdout_tx,
        }
    }

    fn engine(&self) -> &Engine {
        self.engine.as_ref().expect("Engine was not initialized?")
    }

    pub fn start(&mut self) {
        loop {
            let received = self.rx.recv().unwrap();
            match received {
                IncomingCommand::Uci => {
                    self.tx
                        .send(OutgoingCommand::Id("name", "chessatiel"))
                        .unwrap();
                    self.tx
                        .send(OutgoingCommand::Id(
                            "author",
                            "Tim E (https://lichess.org/@/Dragnmn)",
                        ))
                        .unwrap();
                    self.tx.send(OutgoingCommand::UciOk).unwrap()
                }
                IncomingCommand::Debug(_) => {}
                IncomingCommand::IsReady => {
                    if self.engine.is_none() {
                        self.engine = Some(Engine::new());
                    };
                    self.tx.send(OutgoingCommand::ReadyOk).unwrap()
                }
                IncomingCommand::SetOption(_, _) => {}
                IncomingCommand::UciNewGame => {}
                IncomingCommand::Position(mut pos, moves) => {
                    for m_str in moves {
                        let moves = self
                            .engine()
                            .move_generator()
                            .generate_legal_moves_for(&pos);
                        let found = moves.into_iter().find(|m| m.as_uci() == m_str);
                        match found {
                            Some(m) => pos.make_move(&m),
                            None => warn!(
                                "Did not find applicable move for {} in position {}",
                                m_str, pos
                            ),
                        }
                    }
                    debug!("Resulting position: {}", &pos);
                    self.cur_pos = pos
                }
                IncomingCommand::Go => {
                    let m = self
                        .engine()
                        .find_move(&self.cur_pos)
                        .expect("No moves found? Are we in checkmate/stalemate");
                    self.tx.send(OutgoingCommand::BestMove(m.as_uci())).unwrap()
                }
                IncomingCommand::Stop => {}
                IncomingCommand::Quit => break,
            }
        }
    }
}
