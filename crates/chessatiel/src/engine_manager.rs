use crate::brain::Engine;
use beak::{GoPayload, IncomingCommand, InfoPayload, OutgoingCommand};
use guts::{MoveBuffer, Position};
use log::*;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{atomic, Arc};
use std::thread;
use std::time::Duration;

pub struct EngineManager {
    engine: Option<Engine>,
    cur_pos: Position,
    rx: Receiver<IncomingCommand>,
    tx: Sender<OutgoingCommand>,
    running: Arc<AtomicBool>,
}
impl EngineManager {
    pub fn new(stdin_rx: Receiver<IncomingCommand>, stdout_tx: Sender<OutgoingCommand>) -> Self {
        Self {
            engine: None,
            cur_pos: Position::default(),
            rx: stdin_rx,
            tx: stdout_tx,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn engine(&self) -> &Engine {
        self.engine.as_ref().expect("Engine was not initialized?")
    }

    pub fn start(&mut self) {
        loop {
            let received = self.rx.recv().unwrap();
            self.tx
                .send(OutgoingCommand::Info(InfoPayload::String(
                    received.to_string(),
                )))
                .unwrap();
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
                        let tx = self.tx.clone();
                        let running = self.running.clone();
                        let stats = self.engine().statistics().clone();
                        thread::Builder::new()
                            .name("info".to_owned())
                            .spawn(move || {
                                let mut prev = 0;
                                loop {
                                    if running.load(atomic::Ordering::Acquire) {
                                        let cur =
                                            stats.nodes_searched().load(atomic::Ordering::Acquire);
                                        tx.send(OutgoingCommand::Info(InfoPayload::Nps(
                                            (cur - prev) / 5,
                                        )))
                                        .unwrap();
                                        prev = cur;
                                    }
                                    thread::sleep(Duration::from_secs(5))
                                }
                            })
                            .unwrap();
                    };
                    self.tx.send(OutgoingCommand::ReadyOk).unwrap()
                }
                IncomingCommand::SetOption(_, _) => {}
                IncomingCommand::UciNewGame => {}
                IncomingCommand::Position(mut pos, moves) => {
                    for m_str in moves {
                        let mut buf = MoveBuffer::new();
                        let _checked = self
                            .engine()
                            .move_generator()
                            .generate_legal_moves_for(&pos, &mut buf);
                        let found = buf.iter().find(|m| m.as_uci() == m_str);
                        match found {
                            Some(m) => pos.make_move(m),
                            None => warn!(
                                "Did not find applicable move for {} in position {}",
                                m_str, pos
                            ),
                        }
                    }
                    debug!("Resulting position: {}", &pos);
                    self.cur_pos = pos
                }
                IncomingCommand::Go(go_payload) => match go_payload {
                    GoPayload::Perft(_) => todo!(),
                    GoPayload::Depth(d) => {
                        self.running.store(true, atomic::Ordering::Release);
                        let m = self
                            .engine()
                            .search(d, &self.cur_pos)
                            .expect("No moves found? Are we in checkmate/stalemate");
                        self.running.store(false, atomic::Ordering::Release);
                        self.tx
                            .send(OutgoingCommand::BestMove(m.chess_move().as_uci()))
                            .unwrap()
                    }
                    GoPayload::Movetime(_) => {
                        self.tx.send(OutgoingCommand::Info(InfoPayload::String("Ignoring movetime, going depth 5 instead...".to_string()))).unwrap();
                        self.running.store(true, atomic::Ordering::Release);
                        let m = self
                            .engine()
                            .search(5, &self.cur_pos)
                            .expect("No moves found? Are we in checkmate/stalemate");
                        self.running.store(false, atomic::Ordering::Release);
                        self.tx
                            .send(OutgoingCommand::BestMove(m.chess_move().as_uci()))
                            .unwrap()
                    }
                },
                IncomingCommand::Stop => {}
                IncomingCommand::Quit => break,
            }
        }
    }
}
