use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct Statistics {
    nodes_searched: AtomicU64,
    transposition_table_size: AtomicUsize,
    full_transposition_table_hits: AtomicUsize,
    partial_transposition_table_hits: AtomicUsize,
    moves_reordered: AtomicUsize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            nodes_searched: AtomicU64::new(0),
            transposition_table_size: AtomicUsize::new(0),
            full_transposition_table_hits: AtomicUsize::new(0),
            partial_transposition_table_hits: AtomicUsize::new(0),
            moves_reordered: AtomicUsize::new(0),
        }
    }

    pub fn nodes_searched(&self) -> &AtomicU64 {
        &self.nodes_searched
    }

    pub fn transposition_table_size(&self) -> &AtomicUsize {
        &self.transposition_table_size
    }

    pub fn full_transposition_table_hits(&self) -> &AtomicUsize {
        &self.full_transposition_table_hits
    }

    pub fn partial_transposition_table_hits(&self) -> &AtomicUsize {
        &self.partial_transposition_table_hits
    }

    pub fn moves_reordered(&self) -> &AtomicUsize {
        &self.moves_reordered
    }

    pub fn reset(&self) {
        self.nodes_searched.store(0, Ordering::SeqCst);
        self.transposition_table_size.store(0, Ordering::SeqCst);
        self.full_transposition_table_hits
            .store(0, Ordering::SeqCst);
        self.partial_transposition_table_hits
            .store(0, Ordering::SeqCst);
        self.moves_reordered.store(0, Ordering::SeqCst);
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
