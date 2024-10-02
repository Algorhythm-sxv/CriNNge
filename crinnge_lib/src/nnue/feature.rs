use crate::types::*;
use crinnge_bitboards::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Feature {
    pub color: Color,
    pub piece: Piece,
    pub square: Square,
}

impl Feature {
    pub fn index(&self, persp: Color) -> usize {
        const PIECE_OFFSET: usize = 64;
        const COLOR_OFFSET: usize = PIECE_OFFSET * 6;

        let ntm = persp != self.color;
        let square = if persp == White {
            self.square
        } else {
            self.square.flip()
        };

        ntm as usize * COLOR_OFFSET + self.piece as usize * PIECE_OFFSET + *square as usize
    }
}
