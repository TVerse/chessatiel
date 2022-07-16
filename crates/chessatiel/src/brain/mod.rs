use guts::{Move, MoveBuffer, MoveGenerator, Position};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::instrument;

static INNER_INSTANCE: Lazy<EngineInner> = Lazy::new(|| EngineInner {
    move_generator: MoveGenerator::new(),
});

#[derive(Debug)]
struct EngineInner {
    move_generator: MoveGenerator,
}

#[derive(Debug)]
pub struct Engine {
    inner: &'static EngineInner,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            inner: &INNER_INSTANCE,
        }
    }

    #[instrument]
    pub fn get_best_move(&self, position: &Position) -> Move {
        let mut buffer = MoveBuffer::new();
        let _in_check = self
            .inner
            .move_generator
            .generate_legal_moves_for(position, &mut buffer);

        buffer[0].clone()
    }
}
