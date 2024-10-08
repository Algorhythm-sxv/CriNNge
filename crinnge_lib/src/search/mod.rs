pub mod info;
pub mod options;

use std::sync::atomic::Ordering;
use std::thread;

use info::SearchInfo;

use crate::move_sorting::MoveSorter;
use crate::moves::{Move, MoveList, PrincipalVariation};
use crate::types::*;
use crate::{board::Board, thread_data::ThreadData};

pub const MAX_DEPTH: i32 = 128;
pub const MATE_SCORE: i32 = 31_000;
pub const MIN_MATE_SCORE: i32 = MATE_SCORE - MAX_DEPTH;
pub const TB_WIN_SCORE: i32 = 30_000;
pub const MIN_TB_WIN_SCORE: i32 = TB_WIN_SCORE - MAX_DEPTH;
pub const INF: i32 = 32_000;

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

pub trait NodeType {
    const ROOT: bool;
}
pub struct Root;
pub struct NonRoot;

impl NodeType for Root {
    const ROOT: bool = true;
}

impl NodeType for NonRoot {
    const ROOT: bool = false;
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
        info.node_buffer = 0;
        info.stopped.store(false, Ordering::Relaxed);

        // clear leftover PVs from previous searches
        for t in threads_data.iter_mut() {
            t.pv.clear();
        }

        let (t1, rest) = threads_data.split_first_mut().unwrap();
        thread::scope(|s| {
            // spawn helper threads
            for t in rest {
                let board = *self;
                let mut info = *info;

                s.spawn(move || {
                    board.iterative_deepening::<HelperThread>(&mut info, t);
                });
            }

            // main thread work
            self.iterative_deepening::<MainThread>(info, t1);
            info.stopped.store(true, Ordering::Relaxed);
        });

        // select best thread
        let (mut best_thread, rest) = threads_data.split_first().unwrap();
        for t in rest {
            if t.depth_reached == best_thread.depth_reached && t.root_score > best_thread.root_score
            {
                best_thread = t;
            }
            if t.depth_reached > best_thread.depth_reached {
                best_thread = t;
            }
        }

        let best_move = *best_thread.pv.first().unwrap_or_else(|| &legals[0]);

        // reporting to stdout
        if info.stdout {
            println!("bestmove {}", best_move.coords());
            #[cfg(feature = "stats")]
            info.print_stats(best_thread.depth_reached);
        }

        (best_thread.root_score, Some(best_move))
    }

    fn iterative_deepening<M: ThreadType>(&self, info: &mut SearchInfo, t: &mut ThreadData) {
        let mut window = AspirationWindow::default();
        for i in 1..MAX_DEPTH {
            if i > 1 {
                window = AspirationWindow::new_around(t.root_score, info.options.asp_window_init);
            }
            let mut pv = PrincipalVariation::new();
            let score = self.aspiration_window::<M>(&mut pv, info, t, &mut window, i);

            // fixed time, hard time limit or node limit reached somewhere in the main thread
            if info.stopped::<M>() {
                // can't trust results from a partial search, but report accurate statistics for node-determinism
                info.print_depth_report::<M>(t, i);
                break;
            }

            // update thread data for depth report
            t.root_score = score;
            t.depth_reached = i;
            t.pv = pv;

            info.print_depth_report::<M>(t, i);

            // check depth condition in all threads
            if info.time_manager.depth_limit_reached(i) {
                // let other threads run to this depth
                break;
            }

            // check time and node conditions in the main thread
            if M::MAIN_THREAD
                && (info.time_manager.soft_time_limit_reached()
                    || info
                        .time_manager
                        .node_limit_reached(info.global_node_count()))
            {
                // stop the other threads
                info.stop();
                break;
            }
        }
    }

    fn aspiration_window<M: ThreadType>(
        &self,
        pv: &mut PrincipalVariation,
        info: &mut SearchInfo,
        t: &mut ThreadData,
        window: &mut AspirationWindow,
        depth: i32,
    ) -> i32 {
        loop {
            let score = self.negamax::<Root, M>(pv, info, t, window.lower, window.upper, depth, 0);

            // search was stopped partway through
            if info.stopped::<M>() {
                return -INF;
            }

            let score_type = window.test(score);
            match score_type {
                // fail low
                ScoreType::UpperBound => {
                    window.expand_down(info.options.asp_window_scale_percent);
                    #[cfg(feature = "stats")]
                    {
                        info.fail_lows += 1;
                    }
                }
                // fail high
                ScoreType::LowerBound => {
                    window.expand_up(info.options.asp_window_scale_percent);

                    #[cfg(feature = "stats")]
                    {
                        info.fail_highs += 1;
                    }
                }
                // within window
                ScoreType::Exact => return score,
            }

            info.print_aw_fail_report::<M>(depth, score, score_type, &t.pv);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn negamax<R: NodeType, M: ThreadType>(
        &self,
        pv: &mut PrincipalVariation,
        info: &mut SearchInfo,
        t: &mut ThreadData,
        mut alpha: i32,
        beta: i32,
        depth: i32,
        ply: usize,
    ) -> i32 {
        if depth <= 0 {
            let score = self.quiesce::<M>(pv, info, t, alpha, beta, ply);
            return score;
        }

        // check time and node aborts every 1024 nodes on the main thread
        if M::MAIN_THREAD
            && info.inc_nodes()
            && (info
                .time_manager
                .node_limit_reached(info.global_node_count())
                || info.time_manager.hard_time_limit_reached())
        {
            info.stop();
            return -INF;
        }

        // check for aborted search
        if info.stopped::<M>() {
            pv.clear();
            return 0;
        }

        // check for searching too deep
        if ply >= MAX_DEPTH as usize - 1 {
            return self.evaluate(t, ply);
        }

        let pv_node = alpha != beta - 1;

        // probe TT
        let mut tt_move = None;
        let tt_entry = t.tt.get(self.hash());
        if let Some(entry) = tt_entry {
            if !pv_node && entry.depth as i32 >= depth && entry.score_beats_bounds(alpha, beta, ply)
            {
                pv.clear();
                return entry.score.get(ply);
            }
            // TODO: use TT score as static eval when not pruned

            // use the best move saved in the TT for move ordering
            if entry.best_move != Move::NULL {
                tt_move = Some(entry.best_move);
            }
        }

        let mut line = PrincipalVariation::new();
        let in_check = self.in_check();

        info.seldepth = info.seldepth.max(ply + 1);

        if !R::ROOT && self.halfmove_clock() >= 100 {
            // TODO: repetition detection
            pv.clear();
            return randomize_draw_score(info);
        }

        let [mut noisy, mut quiet] = [MoveList::new(); 2];
        let mut move_sorter = MoveSorter::new(tt_move, &mut noisy, &mut quiet);

        let old_alpha = alpha;
        let mut best_score = -INF;
        let mut best_move = None;
        let mut moves_made = 0;

        while let Some((mv, _)) = move_sorter.next(self, t) {
            let mut new = *self;

            if !new.make_move_nnue(mv, t, ply) {
                continue;
            }
            moves_made += 1;

            line.clear();

            let mut score = -INF;

            let search_full_depth_null_window = if moves_made > 1 {
                // TODO: reduced searches
                true
            } else {
                // first moves in non-PV nodes don't get reduced, and later moves in PV nodes that pass the reduced search
                // get re-searched at full depth
                !pv_node || moves_made > 1
            };

            // full depth null window search on later PV moves or non-PV moves that pass the reduced search
            if search_full_depth_null_window {
                score = -new.negamax::<NonRoot, M>(
                    &mut line,
                    info,
                    t,
                    -alpha - 1,
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            }

            // full search on PV first moves and later moves that improve alpha
            if pv_node && (moves_made == 1 || (score > alpha && score < beta)) {
                score = -new.negamax::<NonRoot, M>(
                    &mut line,
                    info,
                    t,
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            }

            if info.stopped::<M>() {
                // can't trust results from stopped searches
                pv.clear();
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                pv.update_with(mv, &line);
                if score > alpha {
                    alpha = score;
                }
                if alpha >= beta {
                    #[cfg(feature = "stats")]
                    {
                        info.fail_highs += 1;
                    }
                    break;
                }
            }
        }

        if moves_made == 0 {
            // no legal moves, checkmate or stalemate
            pv.clear();
            if in_check {
                return -(MATE_SCORE - ply as i32);
            } else {
                return randomize_draw_score(info);
            }
        }

        best_score = best_score.clamp(-MATE_SCORE, MATE_SCORE);

        if alpha != old_alpha {
            // alpha raised, we must have a new PV
            // TODO: history heuristics
        }

        // store search results in TT
        let score_type = if best_score >= beta {
            ScoreType::LowerBound
        } else if best_score > old_alpha {
            ScoreType::Exact
        } else {
            ScoreType::UpperBound
        };

        if best_move.is_none() {
            println!("{}", self.fen());
        }

        t.tt.store(
            self.hash(),
            best_score,
            score_type,
            best_move.expect("TT set with no moves played"),
            depth,
            ply,
        );

        best_score
    }

    fn quiesce<M: ThreadType>(
        &self,
        pv: &mut PrincipalVariation,
        info: &mut SearchInfo,
        t: &mut ThreadData,
        mut alpha: i32,
        beta: i32,
        ply: usize,
    ) -> i32 {
        // check time and node aborts every 1024 nodes on the main thread
        if M::MAIN_THREAD
            && info.inc_nodes()
            && (info
                .time_manager
                .node_limit_reached(info.global_node_count())
                || info.time_manager.hard_time_limit_reached())
        {
            info.stop();
            return -INF;
        }

        // check for aborted search
        if info.stopped::<M>() {
            pv.clear();
            return 0;
        }

        // check for searching too deep
        if ply >= MAX_DEPTH as usize - 1 {
            return self.evaluate(t, ply);
        }

        info.seldepth = info.seldepth.max(ply + 1);
        let in_check = self.in_check();

        // probe TT
        let mut tt_move = Move::NULL;
        let tt_entry = t.tt.get(self.hash());
        if let Some(entry) = tt_entry {
            // TODO: pruning outside of PV, after PVS impl
            // TODO: use TT score as static eval when not pruned

            // use the best move saved in the TT for move ordering
            tt_move = entry.best_move;
        }

        let mut static_eval = self.evaluate(t, ply);

        // if the static eval is too good the opponent won't play into this position
        if static_eval >= beta && !in_check {
            pv.clear();
            #[cfg(feature = "stats")]
            {
                info.fail_highs += 1;
            }
            return static_eval;
        }

        alpha = alpha.max(static_eval);
        let old_alpha = alpha;

        let mut line = PrincipalVariation::new();

        let [mut noisy, mut quiet] = [MoveList::new(); 2];
        self.generate_moves_into(&mut noisy, &mut quiet);

        let mut best_move = None;
        let mut best_score = static_eval;
        let mut moves_made = 0;

        for &mv in noisy.iter_moves() {
            let mut new = *self;
            if !new.make_move_nnue(mv, t, ply) {
                continue;
            }
            moves_made += 1;

            line.clear();

            let score = -new.quiesce::<M>(&mut line, info, t, -beta, -alpha, ply + 1);

            if info.stopped::<M>() {
                pv.clear();
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                pv.update_with(mv, &line);
                if score > alpha {
                    alpha = score;
                }
                if alpha >= beta {
                    #[cfg(feature = "stats")]
                    {
                        info.fail_highs += 1;
                    }
                    break;
                }
            }
        }

        if moves_made == 0 {
            pv.clear();
            if noisy.len() + quiet.len() == 0 {
                // no legal moves, checkmate or stalemate
                if in_check {
                    return -(MATE_SCORE - ply as i32);
                } else {
                    return randomize_draw_score(info);
                }
            }
        }

        best_score = best_score.clamp(-MATE_SCORE, MATE_SCORE);

        // store search results in TT
        let score_type = if best_score >= beta {
            ScoreType::LowerBound
        } else if best_score > old_alpha {
            ScoreType::Exact
        } else {
            ScoreType::UpperBound
        };

        t.tt.store(
            self.hash(),
            best_score,
            score_type,
            best_move.unwrap_or_default(),
            0, // depth
            ply,
        );

        best_score
    }
}

fn randomize_draw_score(info: &SearchInfo) -> i32 {
    4 - (info.global_node_count() as i32 & 7)
}
