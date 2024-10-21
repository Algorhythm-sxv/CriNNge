use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{board::Board, types::*};
use crinnge_bitboards::*;

const FLAGS_MASK: u16 = 0b1100_0000_0000_0000;
const PROMO_FLAG: u16 = FLAGS_MASK;
const CASTLE_FLAG: u16 = 0b1000_0000_0000_0000;
const EP_FLAG: u16 = 0b0100_0000_0000_0000;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Move(pub u16);

impl Move {
    pub const NULL: Self = Self(0);
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
    pub fn from_pair<T: AsRef<str>>(board: &Board, pair: T) -> Self {
        let pair = pair.as_ref();
        let from = Square::from_coord(&pair[0..2]);
        let mut to = Square::from_coord(&pair[2..4]);
        let promo = match pair.chars().nth(4) {
            Some('n') => Some(Knight),
            Some('b') => Some(Bishop),
            Some('r') => Some(Rook),
            Some('q') => Some(Queen),
            _ => None,
        };

        let piece = board.piece_on(from).unwrap_or(Pawn);
        if piece == King {
            // correct for castling
            if to.file() > from.file() && to.file().abs_diff(from.file()) > 1 {
                // kingside castling
                to = board.castles()[board.player()][0].first_square()
            } else if to.file() < from.file() && to.file().abs_diff(from.file()) > 1 {
                // queenside castling
                to = board.castles()[board.player()][1].first_square()
            }
        }

        Self::new(from, to, promo)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MoveListEntry {
    pub mv: Move,
    pub score: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    moves: [MoveListEntry; 218],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [MoveListEntry::default(); 218],
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[self.len] = MoveListEntry { mv, score: 0 };
        self.len += 1;
    }

    pub fn iter_moves(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.len].iter().map(|e| &e.mv)
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn next(&mut self, index: usize) -> Option<MoveListEntry> {
        if index >= self.len {
            return None;
        }
        let mut best_score = self.moves[index].score;
        let mut best_index = index;

        for i in index + 1..self.len {
            let score = self.moves[i].score;
            if score >= best_score {
                best_score = score;
                best_index = i;
            }
        }

        self.moves.swap(index, best_index);

        Some(self.moves[index])
    }
}

impl Deref for MoveList {
    type Target = [MoveListEntry];

    fn deref(&self) -> &Self::Target {
        &self.moves[..self.len]
    }
}

impl DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves[..self.len]
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PrincipalVariation {
    moves: [Move; 218],
    len: usize,
}

impl PrincipalVariation {
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; 218],
            len: 0,
        }
    }

    pub fn update_with(&mut self, mv: Move, rest: &Self) {
        self.moves[0] = mv;

        let new_len = rest.len + 1;
        self.moves[1..new_len].copy_from_slice(&rest.moves[..rest.len]);
        self.len = new_len;
    }

    pub fn clear(&mut self) {
        self.len = 0
    }
}

impl Default for PrincipalVariation {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for PrincipalVariation {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        &self.moves[..self.len]
    }
}

impl Display for PrincipalVariation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|m| m.coords())
                .collect::<Vec<_>>()
                .join(" ")
        )
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
