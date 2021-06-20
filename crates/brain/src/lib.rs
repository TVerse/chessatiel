use guts::{Move, MoveGenerator, Position};
use rand::seq::SliceRandom;

pub struct Engine {
    move_generator: MoveGenerator,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            move_generator: MoveGenerator::new(),
        }
    }

    pub fn move_generator(&self) -> &MoveGenerator {
        &self.move_generator
    }

    pub fn find_move(&self, position: &Position) -> Option<Move> {
        let moves = self.move_generator.generate_legal_moves_for(position);

        moves.choose(&mut rand::thread_rng()).cloned()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
