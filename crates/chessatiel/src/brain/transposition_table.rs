use crate::brain::ResultInfo;
use guts::ZobristHash;
use std::collections::HashMap;

pub struct TranspositionTable {
    // TODO not pub
    pub map: HashMap<ZobristHash, (ResultInfo, usize)>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(1000000),
        }
    }

    pub fn insert(&mut self, key: ZobristHash, value: ResultInfo, depth: usize) -> usize {
        self.map.insert(key, (value, depth));
        self.map.len()
    }

    pub fn get(&self, key: &ZobristHash) -> Option<(ResultInfo, usize)> {
        self.map.get(key).copied()
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}
