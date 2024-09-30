use crate::{nnue::Accumulator, search::MAX_DEPTH};

#[derive(Debug)]
pub struct ThreadData {
    pub accumulators: [[Accumulator; 2]; MAX_DEPTH],
}
