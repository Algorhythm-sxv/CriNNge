use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use crate::timeman::TimeManager;

pub struct SearchInfo<'a> {
    pub time_manager: TimeManager,
    pub stopped: &'a AtomicBool,
    pub global_nodes: &'a AtomicU64,
    pub local_nodes: u64,
    pub seldepth: i32,
    pub stdout: bool,

    #[cfg(feature = "stats")]
    pub fail_highs: u64,
    #[cfg(feature = "stats")]
    pub fail_lows: u64,
    #[cfg(feature = "stats")]
    pub tt_hits: u64,
}

impl<'a> SearchInfo<'a> {
    const MAX_LOCAL_NODES: u64 = 1024;
    pub fn new(stop_signal: &'a AtomicBool, global_nodes: &'a AtomicU64) -> SearchInfo<'a> {
        Self {
            time_manager: TimeManager::new(Instant::now()),
            stopped: stop_signal,
            global_nodes,
            local_nodes: 0,
            seldepth: 0,
            stdout: true,

            #[cfg(feature = "stats")]
            fail_highs: 0,
            #[cfg(feature = "stats")]
            fail_lows: 0,
            #[cfg(feature = "stats")]
            tt_hits: 0,
        }
    }

    pub fn time_manager(mut self, time_manager: TimeManager) -> Self {
        self.time_manager = time_manager;
        self
    }

    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn inc_nodes(&mut self) {
        self.local_nodes += 1;
        if self.local_nodes >= Self::MAX_LOCAL_NODES {
            self.global_nodes
                .fetch_add(self.local_nodes, Ordering::Relaxed);
            self.local_nodes = 0;
        }
    }

    pub fn node_count(&self) -> u64 {
        self.global_nodes.load(Ordering::Relaxed) + self.local_nodes
    }

    #[cfg(feature = "stats")]
    pub fn print_stats(&self, max_depth: i32) {
        println!(
            "info string fail highs: {}\n\
                info string fail lows: {}\n\
                info string TT hits: {}\n\
                info string branching factor: {}",
            self.fail_highs,
            self.fail_lows,
            self.tt_hits,
            (self.global_nodes.load(Ordering::Relaxed) as f64).powf(1.0 / max_depth as f64)
        );
    }
}
