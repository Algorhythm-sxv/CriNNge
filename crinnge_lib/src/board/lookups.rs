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
