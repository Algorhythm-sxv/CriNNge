use crate::board::Board;
use crate::moves::{Move, MoveList};
use crate::thread_data::ThreadData;
use crate::types::*;

const MVV_LVA: [[i16; 6]; 6] = [
    [15, 14, 13, 12, 11, 10], // Pawn capture
    [25, 24, 23, 22, 21, 20], // Knight capture
    [35, 34, 33, 32, 31, 30], // Bishop capture
    [45, 44, 43, 42, 41, 40], // Rook capture
    [55, 54, 53, 52, 51, 50], // Queen capture
    [0, 0, 0, 0, 0, 0],       // King capture (not possible)
];
const BAD_NOISY: i16 = -10100;


#[derive(PartialEq, Eq)]
pub enum MoveGenStage {
    TTMove,
    GenerateMoves,
    GoodNoisies,
    Quiets,
    BadNoisies,
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

    pub fn next(&mut self, board: &Board, t: &ThreadData) -> Option<(Move, MoveGenStage)> {
        if self.stage == TTMove {
            self.stage = GenerateMoves;
            if let Some(mv) = self.tt_move {
                if board.is_pseudolegal(mv) {
                    debug_assert!(
                        board.pseudolegal_moves().contains(&mv),
                        "Illegal TT move passed pseudolegal check: {}, {}, c: {}, ep: {}, promo: {:?}",
                        board.fen(),
                        mv.coords(),
                        mv.is_castling(),
                        mv.is_ep(),
                        mv.promo(),
                    );
                    return Some((mv, TTMove));
                }
            }
        }

        if self.stage == GenerateMoves {
            self.stage = GoodNoisies;
            board.generate_moves_into(self.noisies, self.quiets);
            self.score_noisies(board, t);
        }

        if self.stage == GoodNoisies {
            loop {
                let noisy = self.noisies.next(self.noisy_index);
                self.noisy_index += 1;
                if let Some(noisy) = noisy {
                    // don't repeat the TT move
                    if Some(noisy.mv) == self.tt_move {
                        continue;
                    }
                    // go to quiets once a bad noisy is discovered
                    if noisy.score < 0 {
                        // put this noisy back in the list
                        self.noisy_index -= 1;
                        if !self.noisy_only {
                            self.stage = Quiets;
                            self.score_quiets(board, t);
                        }
                        break;
                    }

                    return Some((noisy.mv, GoodNoisies));
                } else {
                    if !self.noisy_only {
                        self.stage = Quiets;
                        self.score_quiets(board, t);
                    }
                    break;
                }
            }
        }

        if self.stage == Quiets {
            loop {
                let quiet = self.quiets.next(self.quiet_index);
                self.quiet_index += 1;
                if quiet.is_none() {
                    self.stage = BadNoisies;
                    break;
                } else if quiet.map(|e| e.mv) == self.tt_move {
                    continue;
                }
                return Some((quiet.unwrap().mv, Quiets));
            }
        }

        if self.stage == BadNoisies {
            loop {
                let noisy = self.noisies.get(self.noisy_index);
                self.noisy_index += 1;
                if noisy.is_none() {
                    break;
                } else if noisy.map(|e| e.mv) == self.tt_move {
                    continue;
                }

                return Some((noisy.unwrap().mv, BadNoisies));
            }
        }

        None
    }

    fn score_noisies(&mut self, board: &Board, _t: &ThreadData) {
        for noisy in self.noisies.iter_mut() {
            let piece = board.piece_on(noisy.mv.from()).unwrap();
            let capture = board.piece_on(noisy.mv.to()).unwrap_or(Pawn); // promotions may not have a capture
            let mvv_lva = MVV_LVA[capture][piece];

            let bad = !board.see_beats_threshold(noisy.mv, 0);

            noisy.score = BAD_NOISY * bad as i16 + mvv_lva;
        }
    }

    fn score_quiets(&mut self, board: &Board, t: &ThreadData) {
        for quiet in self.quiets.iter_mut() {
            let piece = board.piece_on(quiet.mv.from()).unwrap();

            quiet.score = t.history.get(piece, quiet.mv.to());
        }
    }
}
