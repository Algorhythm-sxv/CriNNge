use crinnge_bitboards::*;

use crate::types::*;

use super::{feature::Feature, Aligned, NNUE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accumulator {
    pub white: Aligned,
    pub black: Aligned,
}

impl Accumulator {
    pub fn new() -> Self {
        Self {
            white: NNUE.feature_bias,
            black: NNUE.feature_bias,
        }
    }

    pub fn apply_into(&self, dst: &mut Accumulator, updates: MoveUpdates) {
        let white_add1_index = updates.adds[0].index(White);
        let white_sub1_index = updates.subs[0].index(White);
        let black_add1_index = updates.adds[0].index(Black);
        let black_sub1_index = updates.subs[0].index(Black);

        match (updates.nadds, updates.nsubs) {
            (1, 1) => {
                add_sub_into(
                    &self.white,
                    &mut dst.white,
                    white_add1_index,
                    white_sub1_index,
                );
                add_sub_into(
                    &self.black,
                    &mut dst.black,
                    black_add1_index,
                    black_sub1_index,
                );
            }
            (1, 2) => {
                let white_sub2_index = updates.subs[1].index(White);
                let black_sub2_index = updates.subs[1].index(Black);
                add_sub2_into(
                    &self.white,
                    &mut dst.white,
                    white_add1_index,
                    [white_sub1_index, white_sub2_index],
                );
                add_sub2_into(
                    &self.black,
                    &mut dst.black,
                    black_add1_index,
                    [black_sub1_index, black_sub2_index],
                );
            }
            (2, 2) => {
                let white_add2_index = updates.adds[1].index(White);
                let white_sub2_index = updates.subs[1].index(White);
                let black_add2_index = updates.adds[1].index(Black);
                let black_sub2_index = updates.subs[1].index(Black);
                add2_sub2_into(
                    &self.white,
                    &mut dst.white,
                    [white_add1_index, white_add2_index],
                    [white_sub1_index, white_sub2_index],
                );
                add2_sub2_into(
                    &self.black,
                    &mut dst.black,
                    [black_add1_index, black_add2_index],
                    [black_sub1_index, black_sub2_index],
                );
            }
            _ => unreachable!(),
        }
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_in_place(acc: &mut Aligned, index: usize) {
    let weights = NNUE.feature_weights[index];

    for (v, w) in acc.iter_mut().zip(weights.iter()) {
        *v += w
    }
}

pub fn sub_in_place(acc: &mut Aligned, index: usize) {
    let weights = NNUE.feature_weights[index];

    for (v, w) in acc.iter_mut().zip(weights.iter()) {
        *v -= w
    }
}

pub fn add_sub_into(src: &Aligned, dst: &mut Aligned, add: usize, sub: usize) {
    let add_weights = NNUE.feature_weights[add];
    let sub_weights = NNUE.feature_weights[sub];

    for ((src, dst), (add, sub)) in src
        .iter()
        .zip(dst.iter_mut())
        .zip(add_weights.iter().zip(sub_weights.iter()))
    {
        *dst = src + add - sub
    }
}

pub fn add_sub2_into(src: &Aligned, dst: &mut Aligned, add: usize, subs: [usize; 2]) {
    let add_weights = NNUE.feature_weights[add];
    let sub1_weights = NNUE.feature_weights[subs[0]];
    let sub2_weights = NNUE.feature_weights[subs[1]];

    for ((src, dst), (add, (sub1, sub2))) in src.iter().zip(dst.iter_mut()).zip(
        add_weights
            .iter()
            .zip(sub1_weights.iter().zip(sub2_weights.iter())),
    ) {
        *dst = src + add - sub1 - sub2;
    }
}

pub fn add2_sub2_into(src: &Aligned, dst: &mut Aligned, adds: [usize; 2], subs: [usize; 2]) {
    let add1_weights = NNUE.feature_weights[adds[0]];
    let add2_weights = NNUE.feature_weights[adds[1]];
    let sub1_weights = NNUE.feature_weights[subs[0]];
    let sub2_weights = NNUE.feature_weights[subs[1]];

    for ((src, dst), ((add1, add2), (sub1, sub2))) in src.iter().zip(dst.iter_mut()).zip(
        add1_weights
            .iter()
            .zip(add2_weights.iter())
            .zip(sub1_weights.iter().zip(sub2_weights.iter())),
    ) {
        *dst = src + add1 + add2 - sub1 - sub2;
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MoveUpdates {
    adds: [Feature; 2],
    subs: [Feature; 2],
    nadds: usize,
    nsubs: usize,
}

impl MoveUpdates {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, color: Color, piece: Piece, square: Square) {
        self.adds[self.nadds] = Feature {
            color,
            piece,
            square,
        };
        self.nadds += 1;
    }
    pub fn sub(&mut self, color: Color, piece: Piece, square: Square) {
        self.subs[self.nsubs] = Feature {
            color,
            piece,
            square,
        };
        self.nsubs += 1;
    }
}
