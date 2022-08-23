use guts::ZobristHash;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PositionHashHistory {
    initial_hash: ZobristHash,
    hashes: Vec<ZobristHash>,
}

impl PositionHashHistory {
    const INITIAL_VEC_CAPACITY: usize = 100;

    pub fn new(initial_hash: ZobristHash) -> Self {
        let hashes = Vec::with_capacity(Self::INITIAL_VEC_CAPACITY);
        Self {
            initial_hash,
            hashes,
        }
    }

    pub fn reset_with(&mut self, hash: ZobristHash) {
        self.hashes = Vec::with_capacity(Self::INITIAL_VEC_CAPACITY);
        self.initial_hash = hash;
    }
    pub fn push(&mut self, hash: ZobristHash) {
        self.hashes.push(hash)
    }

    pub fn pop(&mut self) -> ZobristHash {
        self.hashes.pop().unwrap()
    }

    #[cfg(debug_assertions)]
    pub fn count(&self) -> usize {
        self.hashes.len() + 1
    }

    pub fn is_threefold_repetition(&self) -> bool {
        // Optimization possibilities:
        // skip every other hash (other player)
        // reset when halfmove clock resets (irreversible)
        let latest = *self.hashes.last().unwrap_or(&self.initial_hash);
        let mut count = 0;
        if latest == self.initial_hash {
            count += 1
        }
        for h in self.hashes.iter().rev().copied() {
            if h == latest {
                count += 1
            }
            if count >= 3 {
                return true;
            }
        }

        false
    }
}
