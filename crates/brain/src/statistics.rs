use std::sync::atomic::{AtomicU64, Ordering};

pub struct Statistics {
    nodes_searched: AtomicU64,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            nodes_searched: AtomicU64::new(0),
        }
    }

    pub fn nodes_searched(&self) -> &AtomicU64 {
        &self.nodes_searched
    }

    pub fn reset(&self) {
        self.nodes_searched.store(0, Ordering::SeqCst)
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
