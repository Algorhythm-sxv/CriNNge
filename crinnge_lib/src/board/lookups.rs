use crate::types::*;
use crinnge_bitboards::*;
use crinnge_pregen::*;

#[inline(always)]
pub fn lookup_knight_moves(square: Square) -> BitBoard {
    KNIGHT_TABLE[*square as usize]
}

#[inline(always)]
pub fn lookup_bishop_moves(square: Square, occupied: BitBoard) -> BitBoard {
    SLIDING_ATTACK_TABLE[bishop_attack_index(square, occupied)]
}

#[inline(always)]
pub fn lookup_rook_moves(square: Square, occupied: BitBoard) -> BitBoard {
    SLIDING_ATTACK_TABLE[rook_attack_index(square, occupied)]
}

#[inline(always)]
pub fn lookup_queen_moves(square: Square, occupied: BitBoard) -> BitBoard {
    lookup_bishop_moves(square, occupied) | lookup_rook_moves(square, occupied)
}

#[inline(always)]
pub fn lookup_king_moves(square: Square) -> BitBoard {
    KING_TABLE[*square as usize]
}

#[inline(always)]
pub fn lookup_between(from: Square, to: Square) -> BitBoard {
    BETWEEN[from][to]
}

#[inline(always)]
pub fn zobrist_piece(color: Color, piece: Piece, square: Square) -> u64 {
    let color_offset = 64 * color as usize;
    ZOBRIST_NUMBERS[64 * 2 * piece as usize + color_offset + *square as usize]
}

#[inline(always)]
pub fn zobrist_player() -> u64 {
    ZOBRIST_NUMBERS[64 * 6 * 2]
}

#[inline(always)]
pub fn zobrist_castling(rights: [[BitBoard; 2]; 2]) -> u64 {
    let index: usize = rights
        .iter()
        .flatten()
        .enumerate()
        .map(|(i, b)| (b.is_not_empty() as usize) << i)
        .sum();
    ZOBRIST_NUMBERS[64 * 6 * 2 + 1 + index]
}

#[inline(always)]
pub fn zobrist_ep(mask: BitBoard) -> u64 {
    ZOBRIST_NUMBERS[64 * 6 * 2 + 1 + 16 + mask.first_square().file()]
}
