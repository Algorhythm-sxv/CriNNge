pub mod evaluation;
pub mod fen;
pub mod lookups;
pub mod movegen;
pub mod utils;
mod see;

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
            if mv.promo().is_none() {
                self.pawn_hash ^= zobrist_piece(player, Pawn, to);
            }
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
        }
        if (to.bitboard() & self.castles[!player][0]).is_not_empty() {
            self.castles[!player][0] = BitBoard::empty();
        } else if (to.bitboard() & self.castles[!player][1]).is_not_empty() {
            self.castles[!player][1] = BitBoard::empty()
        }

        // hash new castling rights
        self.hash ^= zobrist_castling(self.castles);

        self.halfmove_clock += 1;
        if piece == Pawn || capture.is_some() {
            self.halfmove_clock = 0;
        }

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
            if self.hash != self.recalculate_hash()
                || self.pawn_hash != self.recalculate_pawn_hash()
            {
                println!("{}, {}", self.fen(), mv.coords());
                false
            } else {
                true
            }
        });

        true
    }

    pub fn make_null_move_only(&mut self) {
        self.hash ^= zobrist_player() ^ zobrist_ep(self.ep_mask);

        self.player = !self.player;
        self.ep_mask = BitBoard::empty();
        self.hash ^= zobrist_ep(self.ep_mask);
        self.halfmove_clock += 1;

        debug_assert!(self.hash == self.recalculate_hash());
    }

    pub fn make_null_move_nnue(&mut self, t: &mut ThreadData, ply: usize) {
        self.make_null_move_only();

        // copy accumulators to next ply
        t.accumulators[ply + 1] = t.accumulators[ply]
    }

    pub fn is_pseudolegal(&self, mv: Move) -> bool {
        // null moves are never legal
        if mv.is_null() {
            return false;
        }

        // piece type on from square
        let Some(piece) = self.piece_on(mv.from()) else {
            // from square empty
            return false;
        };

        let (pieces, enemy_pieces) = (self.occupied[self.player], self.occupied[!self.player]);

        let from = mv.from().bitboard();
        let to = mv.to().bitboard();

        // moving from a square without a friendly piece
        if (from & pieces).is_empty() {
            return false;
        }

        // capturing a friendly piece (while not castling)
        if (to & pieces).is_not_empty() && !(piece == King && mv.is_castling()) {
            return false;
        }

        // ep or promo set while piece isn't a pawn
        if piece != Pawn && (mv.is_ep() || mv.promo().is_some()) {
            return false;
        }

        // castling set while piece isn't a king
        if piece != King && mv.is_castling() {
            return false;
        }

        // piece special cases
        match piece {
            King => {
                // castling
                let rights = self.castles[self.player];
                if (to & (rights[0] | rights[1])).is_not_empty() {
                    // get for full castling legality
                    let kingside = (to & rights[1]).is_not_empty();
                    let castle = rights[kingside as usize];

                    let rook_from = castle.first_square();
                    // clear between rook and king
                    if (lookup_between(mv.from(), rook_from) & self.all_pieces()).is_empty() {
                        const KING_TARGETS: [[Square; 2]; 2] =
                            [[Square::C1, Square::C8], [Square::G1, Square::G8]];
                        let target = KING_TARGETS[kingside as usize][self.player];
                        let enemy_attacks = self.all_attacks(!self.player);
                        // clear and safe between king and king target
                        return (lookup_between(mv.from(), target)
                            & (self.all_pieces() | enemy_attacks))
                            .is_empty();
                    }
                }
            }
            Pawn => {
                // erroneous promotions
                if mv.promo().is_some() && (to & (FIRST_RANK | EIGHTH_RANK)).is_empty() {
                    return false;
                }
                // pushes
                if matches!((mv.to()).abs_diff(*mv.from()), 8 | 16) {
                    return (self.pawn_pushes(mv.from()) & to).is_not_empty();
                } else {
                    // captures
                    return (self.pawn_attack(mv.from(), self.player)
                        & to
                        & (enemy_pieces | self.ep_mask))
                        .is_not_empty();
                }
            }
            _ => {}
        }

        let occupied = self.all_pieces();
        let piece_attacks = match piece {
            Pawn => unreachable!(),
            Knight => lookup_knight_moves(mv.from()),
            Bishop => lookup_bishop_moves(mv.from(), occupied),
            Rook => lookup_rook_moves(mv.from(), occupied),
            Queen => lookup_queen_moves(mv.from(), occupied),
            King => lookup_king_moves(mv.from()),
        };

        (to & piece_attacks & !pieces).is_not_empty()
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
