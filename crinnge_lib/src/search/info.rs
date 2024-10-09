use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::Receiver,
        Mutex,
    },
    time::Instant,
};

use crate::{
    search::{MATE_SCORE, MAX_DEPTH},
    thread_data::ThreadData,
    timeman::TimeManager,
};

use super::{options::SearchOptions, ScoreType, ThreadType};

pub static UCI_QUIT: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug)]
pub struct SearchInfo<'a> {
    pub time_manager: TimeManager,
    pub stopped: &'a AtomicBool,
    pub global_nodes: &'a AtomicU64,
    pub local_nodes: u64,
    pub node_buffer: u64,
    pub seldepth: usize,
    pub stdin: Option<&'a Mutex<Receiver<String>>>,
    pub stdout: bool,
    pub options: SearchOptions,
    #[cfg(feature = "stats")]
    pub fail_highs: u64,
    #[cfg(feature = "stats")]
    pub fail_lows: u64,
    #[cfg(feature = "stats")]
    pub tt_hits: u64,
}

impl<'a> SearchInfo<'a> {
    pub const MAX_LOCAL_NODES: u64 = 1024;
    pub fn new(stop_signal: &'a AtomicBool, global_nodes: &'a AtomicU64) -> SearchInfo<'a> {
        Self {
            time_manager: TimeManager::new(Instant::now()),
            stopped: stop_signal,
            global_nodes,
            local_nodes: 0,
            node_buffer: 0,
            seldepth: 0,
            stdin: None,
            stdout: true,
            options: SearchOptions::default(),

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

    pub fn stdin(mut self, stdin: Option<&'a Mutex<Receiver<String>>>) -> Self {
        self.stdin = stdin;
        self
    }

    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn inc_nodes(&mut self) -> bool {
        self.node_buffer += 1;
        if self.node_buffer >= Self::MAX_LOCAL_NODES {
            self.global_nodes
                .fetch_add(self.node_buffer, Ordering::Relaxed);
            self.local_nodes += self.node_buffer;
            self.node_buffer = 0;
            true
        } else {
            false
        }
    }

    pub fn global_node_count(&self) -> u64 {
        self.global_nodes.load(Ordering::Relaxed) + self.node_buffer
    }

    pub fn local_node_count(&self) -> u64 {
        self.local_nodes + self.node_buffer
    }

    pub fn stopped<M: ThreadType>(&self) -> bool {
        let signal = self.stopped.load(Ordering::Relaxed);
        if signal {
            return true;
        }
        // check for UCI stop/quit in the main thread
        if M::MAIN_THREAD && self.stdin.is_some() {
            let signal = self
                .stdin
                .unwrap()
                .lock()
                .unwrap()
                .try_recv()
                .map_or(false, |s| s.starts_with("stop"));
            if signal || UCI_QUIT.load(Ordering::Relaxed) {
                self.stop();
            }
        }
        false
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }

    pub fn print_depth_report<M: ThreadType>(&self, t: &ThreadData, depth: i32) {
        if M::MAIN_THREAD && self.stdout {
            let nodes = self.global_node_count();
            let elapsed = self.time_manager.elapsed().as_millis() as u64;
            let nps = nodes * 1_000 / elapsed.max(1);

            let mate_plies = MATE_SCORE - t.root_score.abs();
            let score_string = if mate_plies <= MAX_DEPTH {
                format!(
                    "mate {}{}",
                    if t.root_score > 0 { "" } else { "-" },
                    (mate_plies + 1) / 2
                )
            } else {
                format!("cp {}", t.root_score)
            };

            let hash_fill = t.tt.fill();

            println!(
                "info depth {} seldepth {} score {} nodes {} nps {} hashfull {} time {} pv {}",
                depth, self.seldepth, score_string, nodes, nps, hash_fill, elapsed, t.pv
            );
        }
    }

    pub fn print_aw_fail_report<M: ThreadType>(
        &self,
        depth: i32,
        score: i32,
        score_type: ScoreType,
        t: &ThreadData,
    ) {
        if M::MAIN_THREAD && self.stdout {
            let nodes = self.global_node_count();
            let elapsed = self.time_manager.elapsed().as_millis() as u64;
            let nps = nodes * 1_000 / elapsed.max(1);

            let mate_plies = MATE_SCORE - score.abs();
            let score_string = if mate_plies <= MAX_DEPTH {
                format!(
                    "mate {}{}",
                    if score > 0 { "" } else { "-" },
                    (mate_plies + 1) / 2
                )
            } else {
                format!("cp {}", score)
            };

            let score_bound = match score_type {
                ScoreType::Exact => "",
                ScoreType::LowerBound => " lowerbound",
                ScoreType::UpperBound => " upperbound",
            };

            let hash_fill = t.tt.fill();

            println!(
                "info depth {} seldepth {} score {}{} nodes {} nps {} hashfull {} time {} pv {}",
                depth,
                self.seldepth,
                score_string,
                score_bound,
                nodes,
                nps,
                hash_fill,
                elapsed,
                t.pv
            );
        }
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
