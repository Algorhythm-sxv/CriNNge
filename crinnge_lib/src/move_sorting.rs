use crate::board::Board;
use crate::moves::{Move, MoveList};
use crate::thread_data::ThreadData;

#[derive(PartialEq, Eq)]
pub enum MoveGenStage {
    TTMove,
    GenerateMoves,
    Noisies,
    Quiets,
}

use MoveGenStage::*;

pub struct MoveSorter<'a> {
    tt_move: Option<Move>,
    noisies: &'a mut MoveList,
    noisy_index: usize,
    quiets: &'a mut MoveList,
    quiet_index: usize,
    stage: MoveGenStage,
    noisy_only: bool,
}

impl<'a> MoveSorter<'a> {
    pub fn new(tt_move: Option<Move>, noisies: &'a mut MoveList, quiets: &'a mut MoveList) -> Self {
        Self {
            tt_move,
            noisies,
            noisy_index: 0,
            quiets,
            quiet_index: 0,
            stage: TTMove,
            noisy_only: false,
        }
    }

    pub fn noisy_only(mut self) -> Self {
        self.noisy_only = true;
        self
    }

    pub fn next(&mut self, board: &Board, _t: &ThreadData) -> Option<(Move, MoveGenStage)> {
        if self.stage == TTMove {
            self.stage = GenerateMoves;
            if let Some(mv) = self.tt_move {
                if board.is_pseudolegal(mv) {
                    debug_assert!(
                        board.pseudolegal_moves().contains(&mv),
                        "Illegal TT move passed pseudolegal check: {}, {}",
                        board.fen(),
                        mv.coords()
                    );
                    return Some((mv, TTMove));
                }
            }
        }

        if self.stage == GenerateMoves {
            self.stage = Noisies;
            board.generate_moves_into(self.noisies, self.quiets);
        }

        if self.stage == Noisies {
            // TODO: score noisies
            loop {
                let noisy = self.noisies.get(self.noisy_index).map(|m| m.mv);
                self.noisy_index += 1;
                if noisy.is_none() {
                    if !self.noisy_only {
                        self.stage = Quiets;
                    }
                    break;
                } else if noisy == self.tt_move {
                    continue;
                }
                return Some((noisy.unwrap(), Noisies));
            }
        }

        if self.stage == Quiets {
            // TODO: score quiets
            loop {
                let quiet = self.quiets.get(self.quiet_index).map(|m| m.mv);
                self.quiet_index += 1;
                if quiet.is_none() {
                    break;
                } else if quiet == self.tt_move {
                    continue;
                }
                return Some((quiet.unwrap(), Quiets));
            }
        }

        None
    }
}
