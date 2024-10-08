use crinnge_bitboards::Square;

use crate::types::*;

const HISTORY_MAX: i16 = i16::MAX / 2;

#[derive(Copy, Clone, Debug)]
pub struct HistoryTable([[i16; 64]; 6]);

impl HistoryTable {
    pub fn new() -> Self {
        Self([[0; 64]; 6])
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    pub fn get(&self, piece: Piece, to: Square) -> i16 {
        self.0[piece][to]
    }

    pub fn get_mut(&mut self, piece: Piece, to: Square) -> &mut i16 {
        &mut self.0[piece][to]
    }

    pub fn delta(&self, depth: i32) -> i16 {
        (depth * depth) as i16
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_history_bonus(score: &mut i16, delta: i16) {
    *score += (delta as i32 - (delta as i32 * *score as i32) / HISTORY_MAX as i32) as i16;
}

pub fn apply_history_malus(score: &mut i16, delta: i16) {
    *score -= (delta as i32 + (delta as i32 * *score as i32) / HISTORY_MAX as i32) as i16;
}
