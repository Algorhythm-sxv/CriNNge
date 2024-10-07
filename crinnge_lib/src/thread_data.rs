use crate::{
    board::Board, moves::PrincipalVariation, nnue::Accumulator, search::MAX_DEPTH, tt::TTSlice,
};

#[derive(Clone, Debug)]
pub struct ThreadData<'a> {
    pub accumulators: [Accumulator; MAX_DEPTH as usize],
    pub pv: PrincipalVariation,
    pub root_score: i32,
    pub depth_reached: i32,
    pub tt: TTSlice<'a>,
}

impl<'a> ThreadData<'a> {
    pub fn new(board: &Board, tt: TTSlice<'a>) -> ThreadData<'a> {
        let mut data = Self {
            accumulators: [Accumulator::default(); MAX_DEPTH as usize],
            pv: PrincipalVariation::new(),
            root_score: 0,
            depth_reached: 0,
            tt,
        };

        board.refresh_accumulator(&mut data.accumulators[0]);

        data
    }
}
