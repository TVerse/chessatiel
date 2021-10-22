use guts::{Move, MoveBuffer, MoveGenerator, Position};

use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PositionManagerCommand {
    SetPosition(Position, Vec<String>),
    UpdateInitialPosition(Vec<String>),
}

pub struct PositionManager {
    starting_position: Position,
    current_position: Position,
    move_generator: MoveGenerator,
    rx: mpsc::Receiver<PositionManagerCommand>,
    tx: mpsc::Sender<Position>,
}

impl PositionManager {
    pub fn new(
        move_generator: MoveGenerator,
        rx: mpsc::Receiver<PositionManagerCommand>,
        tx: mpsc::Sender<Position>,
    ) -> Self {
        Self {
            starting_position: Position::default(),
            current_position: Position::default(),
            move_generator,
            rx,
            tx,
        }
    }

    pub async fn run(mut self) {
        tokio::spawn(self.handle_events());
    }

    async fn handle_events(mut self) {
        while let Some(c) = self.rx.recv().await {
            match c {
                PositionManagerCommand::SetPosition(p, ms) => {
                    self.starting_position = p.clone();
                    let newpos = Self::update_pos_with_moves(
                        &self.move_generator,
                        self.starting_position.clone(),
                        &ms,
                    );
                    self.tx.send(self.current_position.clone()).await.unwrap();
                }
                PositionManagerCommand::UpdateInitialPosition(ms) => {
                    let newpos = Self::update_pos_with_moves(
                        &self.move_generator,
                        self.starting_position.clone(),
                        &ms,
                    );
                    if newpos != self.current_position {
                        self.current_position = newpos;
                        self.tx.send(self.current_position.clone()).await.unwrap();
                    }
                }
            }
        }
    }

    fn update_pos_with_moves(
        move_generator: &MoveGenerator,
        position: Position,
        ms: &[String],
    ) -> Position {
        // Until I figure out if Lichess only sends updates after a move, this is necessary.
        let mut buf = MoveBuffer::new();
        let mut newpos = position;
        for m in ms {
            move_generator.generate_legal_moves_for(&newpos, &mut buf);
            if let Some(found) = buf.moves.iter().find(|fm| &fm.as_uci() == m) {
                newpos.make_move(found);
            } else {
                panic!(
                    "Lichess returned invalid move? Position {}, move uci {}",
                    newpos, m
                );
            }
        }
        newpos
    }
}
