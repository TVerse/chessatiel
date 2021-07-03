use crate::brain::ResultInfo;
use guts::ZobristHash;

#[derive(Debug, Clone)]
pub struct TranspositionTableEntry {
    result_info: ResultInfo,
    depth: usize,
    hash: ZobristHash,
}

impl TranspositionTableEntry {
    pub fn new(result_info: ResultInfo, depth: usize, hash: ZobristHash) -> Self {
        Self {
            result_info,
            depth,
            hash,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TranspositionTableResult {
    pub result_info: ResultInfo,
    pub depth: usize,
}

impl TranspositionTableResult {
    fn new(result_info: ResultInfo, depth: usize) -> Self {
        Self { result_info, depth }
    }
}

pub struct TranspositionTable {
    table: Vec<Option<TranspositionTableEntry>>,
    num_entries: usize,
}

impl TranspositionTable {
    pub fn new(num_entries: usize) -> Self {
        Self {
            table: vec![None; num_entries],
            num_entries,
        }
    }

    pub fn of_bytes(size_bytes: usize) -> Self {
        let size_of_entry = std::mem::size_of::<Option<TranspositionTableEntry>>();
        let num_entries = size_bytes / size_of_entry;
        Self::new(num_entries)
    }

    pub fn insert(&mut self, transposition_table_entry: TranspositionTableEntry) {
        let index = self.index(transposition_table_entry.hash);
        let old = self.table[index].as_ref();
        if let Some(old) = old {
            if old.depth < transposition_table_entry.depth {
                self.table[index] = Some(transposition_table_entry);
            }
        } else {
            self.table[index] = Some(transposition_table_entry);
        }
    }

    pub fn get(&self, key: ZobristHash) -> Option<TranspositionTableResult> {
        let index = self.index(key);
        let entry = self.table[index].as_ref();
        entry.and_then(|e| {
            if e.hash == key {
                Some(TranspositionTableResult::new(e.result_info, e.depth))
            } else {
                None
            }
        })
    }

    fn index(&self, key: ZobristHash) -> usize {
        (key.0 as usize) % self.num_entries
    }

    pub fn clear(&mut self) {
        self.table = vec![None; self.num_entries]
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::of_bytes(268_435_456) // 256 MiB
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::Centipawn;

    #[test]
    fn get_empty_table() {
        let hash = ZobristHash(0);
        let table = TranspositionTable::new(16);
        assert_eq!(table.get(hash), None)
    }

    #[test]
    fn insert_and_get() {
        let hash = ZobristHash(0);
        let ri = ResultInfo::new(Centipawn::ZERO, None);
        let depth = 5;
        let entry = TranspositionTableEntry::new(ri, depth, hash);
        let result = TranspositionTableResult::new(ri, depth);
        let mut table = TranspositionTable::new(16);
        table.insert(entry);
        assert_eq!(table.get(hash), Some(result))
    }

    #[test]
    fn insert_two_and_get() {
        let mut table = TranspositionTable::new(16);

        let hash_one = ZobristHash(0);
        let ri_one = ResultInfo::new(Centipawn::ZERO, None);
        let depth_one = 5;
        let entry_one = TranspositionTableEntry::new(ri_one, depth_one, hash_one);
        let result_one = TranspositionTableResult::new(ri_one, depth_one);
        table.insert(entry_one);

        let hash_two = ZobristHash(1);
        let ri_two = ResultInfo::new(Centipawn::ZERO, None);
        let depth_two = 6;
        let entry_two = TranspositionTableEntry::new(ri_two, depth_two, hash_two);
        let result_two = TranspositionTableResult::new(ri_two, depth_two);
        table.insert(entry_two);
        assert_eq!(table.get(hash_one), Some(result_one));
        assert_eq!(table.get(hash_two), Some(result_two));
    }

    #[test]
    fn replace_with_larger_depth() {
        let hash = ZobristHash(0);
        let ri = ResultInfo::new(Centipawn::ZERO, None);
        let depth = 5;
        let entry = TranspositionTableEntry::new(ri, depth, hash);
        let result = TranspositionTableResult::new(ri, depth);
        let mut table = TranspositionTable::new(16);
        table.insert(entry);
        assert_eq!(table.get(hash), Some(result));

        let depth = 6;
        let entry = TranspositionTableEntry::new(ri, depth, hash);
        let result = TranspositionTableResult::new(ri, depth);
        table.insert(entry);
        assert_eq!(table.get(hash), Some(result))
    }
}
