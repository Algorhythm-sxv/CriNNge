use crate::{
    moves::{Move, MoveList},
    types::*,
};

use crinnge_bitboards::*;

use super::{lookups::*, Board};

impl Board {
    pub fn generate_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        noisy.clear();
        quiet.clear();
        self.generate_pawn_moves_into(noisy, quiet);
        self.generate_knight_moves_into(noisy, quiet);
        self.generate_bishop_moves_into(noisy, quiet);
        self.generate_rook_moves_into(noisy, quiet);
        self.generate_queen_moves_into(noisy, quiet);
        self.generate_king_moves_into(noisy, quiet);
        if self.castles[self.player] != [BitBoard::empty(); 2] {
            self.generate_castles_into(quiet);
        }
    }
    pub fn generate_pawn_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let pawns = self.pawns[self.player];
        let occupied = self.all_pieces();
        let enemies = self.occupied[!self.player];

        for pawn in pawns {
            let mut moves = BitBoard::empty();
            match self.player {
                Color::White => {
                    // push 1
                    moves |= (pawn.bitboard() << 8) & !occupied;
                    // push 2
                    moves |= ((moves & THIRD_RANK) << 8) & !occupied;
                }
                Color::Black => {
                    // push 1
                    moves |= (pawn.bitboard() >> 8) & !occupied;
                    // push 2
                    moves |= ((moves & SIXTH_RANK) >> 8) & !occupied
                }
            }
            // attacks
            moves |= self.pawn_attack(pawn, self.player) & (enemies | self.ep_mask);

            for to in moves {
                if matches!(to.rank(), 0 | 7) {
                    // all promotions count as noisy moves
                    noisy.push(Move::new(pawn, to, Some(Piece::Queen)));
                    noisy.push(Move::new(pawn, to, Some(Piece::Rook)));
                    noisy.push(Move::new(pawn, to, Some(Piece::Bishop)));
                    noisy.push(Move::new(pawn, to, Some(Piece::Knight)));
                } else if to.bitboard() == self.ep_mask {
                    noisy.push(Move::new_ep(pawn, to));
                } else if self.enemy_on(to) {
                    noisy.push(Move::new(pawn, to, None));
                } else {
                    quiet.push(Move::new(pawn, to, None));
                }
            }
        }
    }
    pub fn generate_knight_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let knights = self.knights[self.player];
        let friendlies = self.occupied[self.player];
        for from in knights {
            for to in lookup_knight_moves(from) & !friendlies {
                let mv = Move::new(from, to, None);
                if self.enemy_on(to) {
                    noisy.push(mv);
                } else {
                    quiet.push(mv);
                }
            }
        }
    }
    pub fn generate_bishop_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let bishops = self.bishops[self.player];
        let friendlies = self.occupied[self.player];
        for from in bishops {
            for to in lookup_bishop_moves(from, self.all_pieces()) & !friendlies {
                let mv = Move::new(from, to, None);
                if self.enemy_on(to) {
                    noisy.push(mv);
                } else {
                    quiet.push(mv);
                }
            }
        }
    }
    pub fn generate_rook_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let rooks = self.rooks[self.player];
        let friendlies = self.occupied[self.player];
        for from in rooks {
            for to in lookup_rook_moves(from, self.all_pieces()) & !friendlies {
                let mv = Move::new(from, to, None);
                if self.enemy_on(to) {
                    noisy.push(mv);
                } else {
                    quiet.push(mv);
                }
            }
        }
    }
    pub fn generate_queen_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let queens = self.queens[self.player];
        let friendlies = self.occupied[self.player];
        for from in queens {
            for to in lookup_queen_moves(from, self.all_pieces()) & !friendlies {
                let mv = Move::new(from, to, None);
                if self.enemy_on(to) {
                    noisy.push(mv);
                } else {
                    quiet.push(mv);
                }
            }
        }
    }
    pub fn generate_king_moves_into(&self, noisy: &mut MoveList, quiet: &mut MoveList) {
        let from = self.kings[self.player].first_square();
        let friendlies = self.occupied[self.player];
        for to in lookup_king_moves(from) & !friendlies {
            let mv = Move::new(from, to, None);
            if self.enemy_on(to) {
                noisy.push(mv);
            } else {
                quiet.push(mv);
            }
        }
    }
    pub fn generate_castles_into(&self, quiet: &mut MoveList) {
        let enemy_attacks = self.all_attacks(!self.player);
        let from = self.kings[self.player].first_square();
        if (enemy_attacks & from.bitboard()).is_not_empty() {
            // in check, castling is illegal
            return;
        }
        let castles = self.castles[self.player];

        const KING_TARGETS: [[Square; 2]; 2] = [[Square::C1, Square::C8], [Square::G1, Square::G8]];
        for (i, castle) in castles.iter().enumerate() {
            if castle.is_not_empty() {
                let rook_from = castle.first_square();
                // clear between rook and king
                if (lookup_between(from, rook_from) & self.all_pieces()).is_empty() {
                    let target = KING_TARGETS[i][self.player];
                    // clear and safe between king and king target
                    if (lookup_between(from, target) & (self.all_pieces() | enemy_attacks))
                        .is_empty()
                    {
                        let mv = Move::new_castle(from, rook_from);
                        quiet.push(mv);
                    }
                }
            }
        }
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut noisy = MoveList::new();
        let mut quiet = MoveList::new();
        self.generate_moves_into(&mut noisy, &mut quiet);

        let mut legals = Vec::new();
        for mv in noisy.iter_moves().chain(quiet.iter_moves()) {
            if self.clone().make_move_only(*mv) {
                legals.push(*mv)
            }
        }

        legals
    }

    pub fn pseudolegal_moves(&self) -> Vec<Move> {
        let mut noisy = MoveList::new();
        let mut quiet = MoveList::new();
        self.generate_moves_into(&mut noisy, &mut quiet);

        let mut legals = Vec::new();
        for mv in noisy.iter_moves().chain(quiet.iter_moves()) {
            legals.push(*mv)
        }

        legals
    }
}
