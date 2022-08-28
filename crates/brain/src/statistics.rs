use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct Statistics {
    pub current_depth: u64,
    pub nodes_searched: u64,
    pub nodes_searched_this_depth: u64,
    pub tt_hits: u64,
}

#[derive(Default)]
struct StatisticsInternal {
    current_depth: AtomicU64,
    nodes_searched: AtomicU64,
    nodes_searched_this_depth: AtomicU64,
    tt_hits: AtomicU64,
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "current depth: {}", self.current_depth)?;
        writeln!(
            f,
            "nodes searched at current depth: {}",
            self.nodes_searched_this_depth
        )?;
        writeln!(f, "nodes searched total: {}", self.nodes_searched)?;
        write!(f, "transposition table hits: {}", self.tt_hits)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct StatisticsHolder {
    stats: StatisticsInternal,
}

impl StatisticsHolder {
    pub fn new() -> Self {
        Self {
            stats: StatisticsInternal::default(),
        }
    }

    pub fn node_searched(&self) {
        self.stats
            .nodes_searched_this_depth
            .fetch_add(1, Ordering::Relaxed);
        self.stats.nodes_searched.fetch_add(1, Ordering::Relaxed);
    }

    pub fn depth_changed(&self, new_depth: u64) {
        self.stats
            .nodes_searched_this_depth
            .store(0, Ordering::Relaxed);
        self.stats.current_depth.store(new_depth, Ordering::Relaxed);
    }

    pub fn tt_hit(&self) {
        let _ = self.stats.tt_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_statistics(&self) -> Statistics {
        let current_depth = self.stats.current_depth.load(Ordering::Relaxed);
        let nodes_searched_this_depth =
            self.stats.nodes_searched_this_depth.load(Ordering::Relaxed);
        let nodes_searched = self.stats.nodes_searched.load(Ordering::Relaxed);
        let tt_hits = self.stats.tt_hits.load(Ordering::Relaxed);
        Statistics {
            current_depth,
            nodes_searched,
            nodes_searched_this_depth,
            tt_hits,
        }
    }
}
