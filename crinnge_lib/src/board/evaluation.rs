use crate::types::*;
use crate::{search::TB_WIN_SCORE, thread_data::ThreadData};

use super::{Board, NNUE};

impl Board {
    pub fn evaluate(&self, t: &mut ThreadData, ply: usize) -> i32 {
        let acc = match self.player {
            White => &t.accumulators[ply].white,
            Black => &t.accumulators[ply].black,
        };
        let eval = NNUE.evaluate(acc);

        // TODO: material scaling
        // TODO: 50mr scaling

        eval.clamp(-TB_WIN_SCORE + 1, TB_WIN_SCORE - 1)
    }
}
