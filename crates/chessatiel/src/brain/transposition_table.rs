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

#[derive(Debug, Copy, Clone)]
pub struct TranspositionTableResult {
    pub result_info: ResultInfo,
    pub depth: usize,
}

pub struct TranspositionTable {
    table: Vec<Option<TranspositionTableEntry>>,
    num_entries: usize,
}

impl TranspositionTable {
    pub fn new(num_entries: usize) -> Self {
        assert!(
            num_entries.is_power_of_two(),
            "The number of entries in the transposition table must be a power of two. Got: {}.",
            num_entries
        );
        Self {
            table: vec![None; num_entries],
            num_entries,
        }
    }

    pub fn insert(&mut self, transposition_table_entry: TranspositionTableEntry) {
        let masked_key = self.mask(transposition_table_entry.hash);
        let old = self.table[masked_key].as_ref();
        if let Some(old) = old {
            if old.depth < transposition_table_entry.depth {
                self.table[masked_key] = Some(transposition_table_entry);
            }
        } else {
            self.table[masked_key] = Some(transposition_table_entry);
        }
    }

    pub fn get(&self, key: ZobristHash) -> Option<TranspositionTableResult> {
        let masked_key = self.mask(key);
        let entry = self.table[masked_key].as_ref();
        entry.and_then(|e| {
            if e.hash == key {
                Some(TranspositionTableResult {
                    result_info: e.result_info,
                    depth: e.depth,
                })
            } else {
                None
            }
        })
    }

    fn mask(&self, key: ZobristHash) -> usize {
        (key.0 as usize) & (self.num_entries - 1)
    }

    pub fn clear(&mut self) {
        self.table = vec![None; self.num_entries]
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        let size_of_entry = std::mem::size_of::<Option<TranspositionTableEntry>>() as u64;
        let mem_info = sys_info::mem_info().unwrap();
        let system_memory = mem_info.total + mem_info.swap_total;
        let available_memory = (system_memory * 8) / 10; // 80%
        let num_entries = available_memory / size_of_entry;
        let adjusted_num_entries = if num_entries.is_power_of_two() {
            num_entries
        } else {
            num_entries.next_power_of_two() >> 1
        };
        Self::new(adjusted_num_entries as usize)
    }
}
