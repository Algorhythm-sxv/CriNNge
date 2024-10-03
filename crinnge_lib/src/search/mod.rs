pub mod info;

use std::sync::atomic::Ordering;
use std::thread;

use info::SearchInfo;

use crate::moves::Move;
use crate::types::*;
use crate::{board::Board, thread_data::ThreadData};

pub const MAX_DEPTH: usize = 128;

pub trait ThreadType {
    const MAIN_THREAD: bool;
}

pub struct MainThread;
pub struct HelperThread;

impl ThreadType for MainThread {
    const MAIN_THREAD: bool = true;
}

impl ThreadType for HelperThread {
    const MAIN_THREAD: bool = false;
}

impl Board {
    pub fn search(
        &self,
        info: &mut SearchInfo,
        threads_data: &mut [ThreadData],
    ) -> (i32, Option<Move>) {
        let legals = self.legal_moves();
        if legals.is_empty() {
            return (0, None);
        }

        if legals.len() == 1 {
            // TODO: special case 1 legal move
        }

        info.global_nodes.store(0, Ordering::Relaxed);
        info.local_nodes = 0;
        info.stopped.store(false, Ordering::Relaxed);

        let (t1, rest) = threads_data.split_first_mut().unwrap();
        thread::scope(|s| {
            // spawn helper threads
            for t in rest {
                let board = *self;
                let info = *info;

                board.iterative_deepening::<HelperThread>(&mut info, t);
            }

            // main thread work
            self.iterative_deepening::<MainThread>(info, t1);
            info.stopped.store(true, Ordering::Relaxed);
        });

        // select best thread
        let (mut best_thread, rest) = threads_data.split_first().unwrap();
        for t in rest {
            if t.depth_reached > best_thread.depth_reached {
                best_thread = t;
            } else if t.depth_reached == best_thread.depth_reached
                && t.root_score > best_thread.root_score
            {
                best_thread = t;
            }
        }

        let best_move = *best_thread.pv.get(0).unwrap_or_else(|| &legals[0]);

        // reporting to stdout
        if info.stdout {
            println!("bestmove {}", best_move.coords());
            #[cfg(feature = "stats")]
            info.print_stats(best_thread.depth_reached);
        }

        (best_thread.root_score, Some(best_move))
    }
}
