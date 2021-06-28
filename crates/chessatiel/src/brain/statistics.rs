use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct Statistics {
    nodes_searched: AtomicU64,
    transposition_table_size: AtomicUsize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            nodes_searched: AtomicU64::new(0),
            transposition_table_size: AtomicUsize::new(0),
        }
    }

    pub fn nodes_searched(&self) -> &AtomicU64 {
        &self.nodes_searched
    }
    pub fn transposition_table_size(&self) -> &AtomicUsize {
        &self.transposition_table_size
    }

    pub fn reset(&self) {
        self.nodes_searched.store(0, Ordering::SeqCst);
        self.transposition_table_size.store(0, Ordering::SeqCst);
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
