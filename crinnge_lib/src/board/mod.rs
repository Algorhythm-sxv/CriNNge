pub mod fen;
pub mod lookups;
pub mod movegen;
pub mod utils;
pub mod evaluation;

use crinnge_bitboards::*;
use feature::Feature;
use lookups::*;

use crate::{moves::*, nnue::*, thread_data::ThreadData, types::*};

#[derive(Copy, Clone, Debug)]
pub struct Board {
    pawns: [BitBoard; 2],
    knights: [BitBoard; 2],
    bishops: [BitBoard; 2],
    rooks: [BitBoard; 2],
    queens: [BitBoard; 2],
    kings: [BitBoard; 2],
    occupied: [BitBoard; 2],
    castles: [[BitBoard; 2]; 2],
    player: Color,
    ep_mask: BitBoard,
    halfmove_clock: u8,
    fullmove_count: u16,
    hash: u64,
    pawn_hash: u64,
}

impl Board {
    pub fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn empty() -> Self {
        Self {
            pawns: [BitBoard::empty(); 2],
            knights: [BitBoard::empty(); 2],
            bishops: [BitBoard::empty(); 2],
            rooks: [BitBoard::empty(); 2],
            queens: [BitBoard::empty(); 2],
            kings: [BitBoard::empty(); 2],
            occupied: [BitBoard::empty(); 2],
            castles: [[BitBoard::empty(); 2]; 2],
            player: White,
            ep_mask: BitBoard::empty(),
            halfmove_clock: 0,
            fullmove_count: 1,
            hash: 0,
            pawn_hash: 0,
        }
    }

    pub fn move_piece(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
        let pieces = match piece {
            Pawn => &mut self.pawns[color],
            Knight => &mut self.knights[color],
            Bishop => &mut self.bishops[color],
            Rook => &mut self.rooks[color],
            Queen => &mut self.queens[color],
            King => &mut self.kings[color],
        };
        *pieces ^= from.bitboard() | to.bitboard();
        self.occupied[color] ^= from.bitboard() | to.bitboard();
        self.hash ^= zobrist_piece(color, piece, from) ^ zobrist_piece(color, piece, to);
    }

    pub fn xor_piece(&mut self, color: Color, piece: Piece, on: Square) {
        let pieces = match piece {
            Pawn => &mut self.pawns[color],
            Knight => &mut self.knights[color],
            Bishop => &mut self.bishops[color],
            Rook => &mut self.rooks[color],
            Queen => &mut self.queens[color],
            King => &mut self.kings[color],
        };
        *pieces ^= on.bitboard();
        self.occupied[color] ^= on.bitboard();
        self.hash ^= zobrist_piece(color, piece, on);
    }

    pub fn make_move_nnue(&mut self, mv: Move, t: &mut ThreadData, ply: usize) -> bool {
        let mut updates = MoveUpdates::new();
        if !self._make_move(mv, &mut updates) {
            return false;
        }
        let (front, back) = t.accumulators.split_at_mut(ply + 1);
        let before = front.last_mut().unwrap();
        let after = back.first_mut().unwrap();

        before.apply_into(after, updates);

        debug_assert!({
            let mut test = Accumulator::new();
            self.refresh_accumulator(&mut test);

            test == t.accumulators[ply + 1]
        });

        true
    }

    pub fn make_move_only(&mut self, mv: Move) -> bool {
        self._make_move(mv, &mut MoveUpdates::new())
    }

    fn _make_move(&mut self, mv: Move, updates: &mut MoveUpdates) -> bool {
        let from = mv.from();
        let to = mv.to();
        let player = self.player;

        let (piece, capture) = if mv.is_castling() {
            (King, None)
        } else if mv.is_ep() {
            (Pawn, None)
        } else {
            (self.piece_on(from).unwrap(), self.piece_on(to))
        };

        // move the piece normally and remove the normally captured piece
        if !mv.is_castling() {
            updates.sub(player, piece, from);
            if let Some(promo) = mv.promo() {
                self.xor_piece(player, Pawn, from);
                self.xor_piece(player, promo, to);
                updates.add(player, promo, to);
            } else {
                self.move_piece(player, piece, from, to);
                updates.add(player, piece, to);
            }

            if let Some(capture) = capture {
                self.xor_piece(!player, capture, to);
                updates.sub(!player, capture, to);
                if capture == Pawn {
                    self.pawn_hash ^= zobrist_piece(!player, Pawn, to);
                }
            }
        }

        // special cases
        if mv.is_ep() {
            // remove the ep captured pawn
            let target = self.ep_mask.ishift(if player == White { -8 } else { 8 });
            self.pawns[!player] ^= target;
            self.occupied[!player] ^= target;
            updates.sub(!player, Pawn, target.first_square());
            self.hash ^= zobrist_piece(!player, Pawn, target.first_square());
            self.pawn_hash ^= zobrist_piece(!player, Pawn, target.first_square());
        } else if mv.is_castling() {
            // find the destination square
            const DESTS: [[(Square, Square); 2]; 2] = [
                [(Square::C1, Square::D1), (Square::G1, Square::F1)],
                [(Square::C8, Square::D8), (Square::G8, Square::F8)],
            ];
            let kingside = from.file() < to.file();
            let dest = DESTS[player][kingside as usize];

            // move the king and rook
            self.move_piece(player, King, from, dest.0);
            updates.sub(player, King, from);
            updates.add(player, King, dest.0);
            self.move_piece(player, Rook, to, dest.1);
            updates.sub(player, Rook, to);
            updates.add(player, Rook, dest.1);
        }

        // update pawn hash for pawn moves
        if piece == Pawn {
            self.pawn_hash ^= zobrist_piece(player, Pawn, from);
            self.pawn_hash ^= zobrist_piece(player, Pawn, to);
        }

        // clear ep from pawn hash
        self.hash ^= zobrist_ep(self.ep_mask);

        // double pawn push
        if piece == Pawn && from.abs_diff(*to) == 16 {
            let ep_square = to.offset(0, if player == White { -1 } else { 1 });
            self.ep_mask = ep_square.bitboard();
        } else {
            // clear ep for all other moves
            self.ep_mask = BitBoard::empty();
        }

        // re-add new ep hash
        self.hash ^= zobrist_ep(self.ep_mask);

        // clear current castling rights
        self.hash ^= zobrist_castling(self.castles);

        // update castling rights
        if piece == King {
            self.castles[player] = [BitBoard::empty(); 2];
        }
        if (from.bitboard() & self.castles[player][0]).is_not_empty() {
            self.castles[player][0] = BitBoard::empty()
        } else if (from.bitboard() & self.castles[player][1]).is_not_empty() {
            self.castles[player][1] = BitBoard::empty()
        } else if (to.bitboard() & self.castles[!player][0]).is_not_empty() {
            self.castles[!player][0] = BitBoard::empty();
        } else if (to.bitboard() & self.castles[!player][1]).is_not_empty() {
            self.castles[!player][1] = BitBoard::empty()
        }

        // hash new castling rights
        self.hash ^= zobrist_castling(self.castles);

        self.halfmove_clock += 1;
        if player == Black {
            self.fullmove_count += 1;
        }
        self.player = !self.player;
        self.hash ^= zobrist_player();

        // check if king is attacked
        let attacks = self.all_attacks(!player);
        if (self.kings[player] & attacks).is_not_empty() {
            return false;
        }

        debug_assert!({
            self.hash == self.recalculate_hash() && self.pawn_hash == self.recalculate_pawn_hash()
        });

        true
    }

    pub fn refresh_accumulator(&self, acc: &mut Accumulator) {
        let mut new = Accumulator::new();

        for color in [White, Black] {
            for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
                let pieces = self.pieces(piece)[color];
                for square in pieces {
                    // white-relative accumulator
                    add_in_place(
                        &mut new.white,
                        Feature {
                            color,
                            piece,
                            square,
                        }
                        .index(White),
                    );
                    // black-relative accumulator
                    add_in_place(
                        &mut new.black,
                        Feature {
                            color,
                            piece,
                            square,
                        }
                        .index(Black),
                    );
                }
            }
        }

        *acc = new
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
