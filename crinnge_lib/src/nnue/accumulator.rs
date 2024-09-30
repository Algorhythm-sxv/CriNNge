use std::ops::{Index, IndexMut};

use crinnge_bitboards::*;

use crate::types::*;

use super::{HIDDEN_SIZE, NNUE};

const PIECE_OFFSET: usize = 64;
const COLOR_OFFSET: usize = PIECE_OFFSET * 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN_SIZE],
}

impl Accumulator {
    pub fn new() -> Self {
        NNUE.feature_bias
    }
    pub fn apply(&mut self, output: &mut Self, update: MoveUpdates) {
        match (update.nadds, update.nsubs) {
            (1, 1) => self.feature_add_sub(output, update.adds[0], update.subs[0]),
            (1, 2) => self.feature_add_sub2(output, update.adds[0], update.subs),
            (2, 2) => self.feature_add2_sub2(output, update.adds, update.subs),
            _ => unreachable!(),
        }
    }
    pub fn feature_add_in_place(&mut self, stm: bool, piece: Piece, square: Square) {
        let feature_index =
            COLOR_OFFSET * stm as usize + PIECE_OFFSET * piece as usize + *square as usize;
        let feature_weights = NNUE.feature_weights[feature_index];
        for i in 0..HIDDEN_SIZE {
            self[i] += feature_weights[i];
        }
    }
    pub fn feature_sub_in_place(&mut self, stm: bool, piece: Piece, square: Square) {
        let feature_index =
            COLOR_OFFSET * stm as usize + PIECE_OFFSET * piece as usize + *square as usize;
        let feature_weights = NNUE.feature_weights[feature_index];
        for i in 0..HIDDEN_SIZE {
            self[i] -= feature_weights[i];
        }
    }
    fn feature_add_sub(&mut self, output: &mut Self, add_index: usize, sub_index: usize) {
        let add_weights = NNUE.feature_weights[add_index];
        let sub_weights = NNUE.feature_weights[sub_index];
        for i in 0..HIDDEN_SIZE {
            output[i] = self[i] + add_weights[i] - sub_weights[i];
        }
    }
    fn feature_add_sub2(&mut self, output: &mut Self, add_index: usize, subs: [usize; 2]) {
        let add_weights = NNUE.feature_weights[add_index];
        let sub1_weights = NNUE.feature_weights[subs[0]];
        let sub2_weights = NNUE.feature_weights[subs[1]];
        for i in 0..HIDDEN_SIZE {
            output[i] = self[i] + add_weights[i] - sub1_weights[i] - sub2_weights[i];
        }
    }
    fn feature_add2_sub2(&mut self, output: &mut Self, adds: [usize; 2], subs: [usize; 2]) {
        let add1_weights = NNUE.feature_weights[adds[0]];
        let add2_weights = NNUE.feature_weights[adds[1]];
        let sub1_weights = NNUE.feature_weights[subs[0]];
        let sub2_weights = NNUE.feature_weights[subs[1]];
        for i in 0..HIDDEN_SIZE {
            output[i] =
                self[i] + add1_weights[i] + add2_weights[i] - sub1_weights[i] - sub2_weights[i];
        }
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Accumulator {
    type Output = i16;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vals[index]
    }
}

impl IndexMut<usize> for Accumulator {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vals[index]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MoveUpdates {
    adds: [usize; 2],
    subs: [usize; 2],
    nadds: u8,
    nsubs: u8,
}

impl MoveUpdates {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, color: Color, piece: Piece, square: Square) {
        self.adds[self.nadds as usize] =
            color as usize * COLOR_OFFSET + piece as usize * PIECE_OFFSET + *square as usize;
        self.nadds += 1;
    }
    pub fn sub(&mut self, color: Color, piece: Piece, square: Square) {
        self.subs[self.nsubs as usize] =
            color as usize * COLOR_OFFSET + piece as usize * PIECE_OFFSET + *square as usize;
        self.nsubs += 1;
    }
}
