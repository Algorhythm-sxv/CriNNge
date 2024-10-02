use crate::{nnue::Accumulator, search::MAX_DEPTH};

#[derive(Debug)]
pub struct ThreadData {
    pub accumulators: [Accumulator; MAX_DEPTH],
}

impl ThreadData {
    pub fn new() -> Self {
        Self {
            accumulators: [Accumulator::default(); MAX_DEPTH]
        }
    }
}

impl Default for ThreadData {
    fn default() -> Self {
        Self::new()
    }
}