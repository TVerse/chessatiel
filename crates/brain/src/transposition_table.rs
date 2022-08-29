use crate::CentipawnScore;
use guts::{Move, ZobristHash};
use crate::evaluator::ScoreBound;

struct TTKey(u16);

impl From<ZobristHash> for TTKey {
    fn from(zh: ZobristHash) -> Self {
        Self(zh.0 as u16)
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
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        let mut inner = Vec::with_capacity(1 << 16);
        for _ in 0..inner.capacity() {
            inner.push(None)
        }
        Self { inner }
    }

    pub fn get(&self, hash: ZobristHash) -> Option<&TTEntry> {
        self.inner[TTKey::from(hash).0 as usize].as_ref()
    }

    pub fn set(&mut self, entry: TTEntry) {
        let hash = entry.hash;
        self.inner[TTKey::from(hash).0 as usize] = Some(entry)
    }
}
