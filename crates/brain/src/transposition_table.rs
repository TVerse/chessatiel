use crate::evaluator::ScoreBound;
use crate::CentipawnScore;
use guts::{Move, ZobristHash};
use log::info;

#[derive(Debug, Copy, Clone)]
struct TTKey(u64);

impl From<ZobristHash> for TTKey {
    fn from(zh: ZobristHash) -> Self {
        Self(zh.0)
    }
}

#[derive(Debug)]
pub struct TTEntry {
    pub hash: ZobristHash,
    pub depth: u16,
    pub score: CentipawnScore,
    pub bound: ScoreBound,
    pub m: Option<Move>,
}

pub struct TranspositionTable {
    inner: Vec<Option<TTEntry>>,
    mask: u64,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        let max_size_bytes = 16 * 1024 * 1024;
        Self::new(max_size_bytes)
    }
}

impl TranspositionTable {
    pub fn new(max_size_bytes: u64) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>() as u64;
        let ideal_entries = max_size_bytes / entry_size;
        let table_entries = 1 << (63 - ideal_entries.leading_zeros() as u64);
        let mask = table_entries - 1;
        let mut inner = Vec::with_capacity(table_entries as usize);
        for _ in 0..table_entries {
            inner.push(None)
        }
        info!("Initializing transposition table with {table_entries} entries, ({table_size_bytes} bytes total, {entry_size} bytes per entry)", table_size_bytes = table_entries * entry_size);
        Self { inner, mask }
    }

    pub fn get(&self, hash: ZobristHash) -> Option<&TTEntry> {
        self.inner[self.get_index(TTKey::from(hash))].as_ref()
    }

    pub fn set(&mut self, entry: TTEntry) {
        let hash = entry.hash;
        let idx = self.get_index(TTKey::from(hash));
        self.inner[idx] = Some(entry)
    }

    fn get_index(&self, key: TTKey) -> usize {
        (key.0 & self.mask) as usize
    }
}
