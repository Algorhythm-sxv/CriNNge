use crate::types::*;
use crinnge_bitboards::{BitBoard, Square, NOT_A_FILE, NOT_H_FILE};

use super::{lookups::*, Board};

impl Board {
    #[inline(always)]
    pub fn all_pieces(&self) -> BitBoard {
        self.occupied[0] | self.occupied[1]
    }

    pub fn all_attacks(&self, color: Color) -> BitBoard {
        let occupied = self.all_pieces();
        self.pawn_attacks(color)
            | self.knight_attacks(color)
            | self.bishop_attacks(color, occupied)
            | self.rook_attacks(color, occupied)
            | self.queen_attacks(color, occupied)
            | self.king_attacks(color)
    }

    pub fn pawn_attacks(&self, color: Color) -> BitBoard {
        let pawns = self.pawns[color];
        match color {
            White => ((pawns & NOT_A_FILE) << 7) | ((pawns & NOT_H_FILE) << 9),
            Black => ((pawns & NOT_A_FILE) >> 9) | ((pawns & NOT_H_FILE) >> 7),
        }
    }
    pub fn knight_attacks(&self, color: Color) -> BitBoard {
        let mut attacks = BitBoard::empty();
        for knight in self.knights[color] {
            attacks |= lookup_knight_moves(knight);
        }
        attacks
    }
    pub fn bishop_attacks(&self, color: Color, occupied: BitBoard) -> BitBoard {
        let mut attacks = BitBoard::empty();
        for bishop in self.bishops[color] {
            attacks |= lookup_bishop_moves(bishop, occupied);
        }
        attacks
    }
    pub fn rook_attacks(&self, color: Color, occupied: BitBoard) -> BitBoard {
        let mut attacks = BitBoard::empty();
        for rook in self.rooks[color] {
            attacks |= lookup_rook_moves(rook, occupied);
        }
        attacks
    }
    pub fn queen_attacks(&self, color: Color, occupied: BitBoard) -> BitBoard {
        let mut attacks = BitBoard::empty();
        for queen in self.queens[color] {
            attacks |= lookup_queen_moves(queen, occupied);
        }
        attacks
    }
    pub fn king_attacks(&self, color: Color) -> BitBoard {
        lookup_king_moves(self.kings[color].first_square())
    }

    pub fn pawn_attack(&self, pawn: Square, color: Color) -> BitBoard {
        let pawn = pawn.bitboard();
        match color {
            White => ((pawn & NOT_A_FILE) << 7) | ((pawn & NOT_H_FILE) << 9),
            Black => ((pawn & NOT_A_FILE) >> 9) | ((pawn & NOT_H_FILE) >> 7),
        }
    }

    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        let mask = sq.bitboard();
        if (self.all_pieces() & mask).is_empty() {
            None
        } else if ((self.pawns[White] | self.pawns[Black]) & mask).is_not_empty() {
            Some(Pawn)
        } else if ((self.knights[White]
            | self.knights[Black]
            | self.bishops[White]
            | self.bishops[Black])
            & mask)
            .is_not_empty()
        {
            if ((self.knights[White] | self.knights[Black]) & mask).is_not_empty() {
                Some(Knight)
            } else {
                Some(Bishop)
            }
        } else if ((self.rooks[White] | self.rooks[Black]) & mask).is_not_empty() {
            Some(Rook)
        } else if ((self.queens[White] | self.queens[Black]) & mask).is_not_empty() {
            Some(Queen)
        } else {
            Some(King)
        }
    }

    pub fn contents(&self) -> ([[BitBoard; 2]; 7], Color) {
        (
            [
                self.pawns,
                self.knights,
                self.bishops,
                self.rooks,
                self.queens,
                self.kings,
                self.occupied,
            ],
            self.player,
        )
    }

    pub fn pieces(&self, piece: Piece) -> [BitBoard; 2] {
        match piece {
            Pawn => self.pawns,
            Knight => self.knights,
            Bishop => self.bishops,
            Rook => self.rooks,
            Queen => self.queens,
            King => self.kings,
        }
    }

    pub fn player(&self) -> Color {
        self.player
    }

    pub fn recalculate_hash(&self) -> u64 {
        let mut hash = 0;

        for color in [White, Black] {
            for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
                let pieces = self.pieces(piece);
                for square in pieces[color] {
                    hash ^= zobrist_piece(color, piece, square);
                }
            }
        }

        hash ^= zobrist_ep(self.ep_mask);
        hash ^= zobrist_castling(self.castles);
        if self.player == Black {
            hash ^= zobrist_player();
        }

        hash
    }

    pub fn recalculate_pawn_hash(&self) -> u64 {
        let mut hash = 0;

        for color in [White, Black] {
            for square in self.pawns[color] {
                hash ^= zobrist_piece(color, Pawn, square);
            }
        }

        hash
    }
}
