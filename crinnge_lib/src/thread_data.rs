use crate::{moves::PrincipalVariation, nnue::Accumulator, search::MAX_DEPTH};

#[derive(Clone, Debug)]
pub struct ThreadData<'a> {
    pub accumulators: [Accumulator; MAX_DEPTH],
    pub pv: PrincipalVariation,
    pub root_score: i32,
    pub depth_reached: i32,
    pub _tt: &'a [u32],
}

impl<'a> ThreadData<'a> {
    pub fn new() -> ThreadData<'a> {
        Self {
            accumulators: [Accumulator::default(); MAX_DEPTH],
            pv: PrincipalVariation::new(),
            root_score: 0,
            depth_reached: 0,
            _tt: &[],
        }
    }
}

impl<'a> Default for ThreadData<'a> {
    fn default() -> Self {
        Self::new()
    }
}
