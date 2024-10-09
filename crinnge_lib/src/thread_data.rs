use crate::{
    board::Board,
    historytables::*,
    moves::{Move, MoveList, PrincipalVariation},
    nnue::Accumulator,
    search::MAX_DEPTH,
    tt::TTSlice,
};

#[derive(Clone, Debug)]
pub struct ThreadData<'a> {
    pub accumulators: [Accumulator; MAX_DEPTH as usize],
    pub pv: PrincipalVariation,
    pub root_score: i32,
    pub depth_reached: i32,
    pub tt: TTSlice<'a>,
    pub history: HistoryTable,
    pub nmp_enabled: bool,
}

impl<'a> ThreadData<'a> {
    pub fn new(board: &Board, tt: TTSlice<'a>) -> ThreadData<'a> {
        let mut data = Self {
            accumulators: [Accumulator::default(); MAX_DEPTH as usize],
            pv: PrincipalVariation::new(),
            root_score: 0,
            depth_reached: 0,
            tt,
            history: HistoryTable::new(),
            nmp_enabled: true,
        };

        board.refresh_accumulator(&mut data.accumulators[0]);

        data
    }

    pub fn update_quiet_histories(
        &mut self,
        board: &Board,
        depth: i32,
        bonus_quiet: Move,
        quiets_tried: &MoveList,
    ) {
        let delta = self.history.delta(depth);
        // bonus for best quiet
        apply_history_bonus(
            self.history.get_mut(
                board.piece_on(bonus_quiet.from()).unwrap(),
                bonus_quiet.to(),
            ),
            delta,
        );

        // malus for quiets that weren't the best
        for &malus_quiet in quiets_tried.iter_moves() {
            // don't punish the best move!
            if malus_quiet == bonus_quiet {
                continue;
            }
            apply_history_malus(
                self.history.get_mut(
                    board.piece_on(malus_quiet.from()).unwrap(),
                    malus_quiet.to(),
                ),
                delta,
            );
        }
    }
    
    pub fn reset(&mut self) {
        self.pv.clear();
        self.history.clear();
        self.nmp_enabled = true;
    }
}
