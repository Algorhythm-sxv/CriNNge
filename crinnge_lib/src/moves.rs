use crate::types::*;
use crinnge_bitboards::*;

const FLAGS_MASK: u16 = 0b1100_0000_0000_0000;
const PROMO_FLAG: u16 = FLAGS_MASK;
const CASTLE_FLAG: u16 = 0b1000_0000_0000_0000;
const EP_FLAG: u16 = 0b0100_0000_0000_0000;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Move(pub u16);

impl Move {
    const NULL: Self = Self(0);
    pub fn new(from: Square, to: Square, promo: Option<Piece>) -> Self {
        let mut inner = *from as u16;
        inner |= (*to as u16) << 6;
        if let Some(promo) = promo {
            inner |= PROMO_FLAG;
            inner |= (promo as u16 - 1) << 12;
        }
        Self(inner)
    }
    pub fn new_castle(from: Square, to: Square) -> Self {
        let mut inner = *from as u16;
        inner |= (*to as u16) << 6;
        inner |= CASTLE_FLAG;

        Self(inner)
    }
    pub fn new_ep(from: Square, to: Square) -> Self {
        let mut inner = *from as u16;
        inner |= (*to as u16) << 6;
        inner |= EP_FLAG;

        Self(inner)
    }
    pub fn from(&self) -> Square {
        Square::from(self.0 & 0b111111)
    }
    pub fn to(&self) -> Square {
        Square::from((self.0 >> 6) & 0b111111)
    }
    pub fn promo(&self) -> Option<Piece> {
        if self.0 & FLAGS_MASK == PROMO_FLAG {
            Some(Piece::from(((self.0 >> 12 & 0b11) as u8) + 1))
        } else {
            None
        }
    }
    pub fn is_castling(&self) -> bool {
        self.0 & FLAGS_MASK == CASTLE_FLAG
    }
    pub fn is_ep(&self) -> bool {
        self.0 & FLAGS_MASK == EP_FLAG
    }
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
    pub fn coords(&self) -> String {
        if self.is_castling() {
            let black = self.from().rank() == 7;
            let kingside = self.to().file() > self.from().file();
            const CASTLES: [[&str; 2]; 2] = [["e1c1", "e1g1"], ["e8c8", "e8g8"]];
            return CASTLES[black as usize][kingside as usize].to_string();
        }
        let from = self.from().coord();
        let to = self.to().coord();
        let promo = if let Some(promo) = self.promo() {
            match promo {
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Rook => "r",
                Piece::Queen => "q",
                _ => unreachable!(),
            }
        } else {
            ""
        };

        format!("{from}{to}{promo}")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    moves: [Move; 218],
    len: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 218],
            len: 0,
        }
    }
    pub fn push(&mut self, mv: Move) {
        self.moves[self.len as usize] = mv;
        self.len += 1;
    }
    pub fn slice(&self) -> &[Move] {
        &self.moves[..self.len as usize]
    }
    
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crinnge_bitboards::Square;

    use super::{Move, Piece};

    #[test]
    fn test_move_packing() {
        let mv = Move::new(Square::A7, Square::A8, Some(Piece::Rook));
        assert!(mv.from() == Square::A7);
        assert!(mv.to() == Square::A8);
        assert!(mv.promo() == Some(Piece::Rook));
        assert!(!mv.is_castling());
        assert!(!mv.is_ep());

        let mv = Move::new_castle(Square::E1, Square::G1);
        assert!(mv.is_castling());

        let mv = Move::new_ep(Square::E5, Square::D6);
        assert!(mv.is_ep());
    }
}
