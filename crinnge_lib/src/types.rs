use std::ops::{Div, Index, IndexMut, Not};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Color {
    #[default]
    White = 0,
    Black = 1,
}

impl<T, const N: usize> Index<Color> for [T; N] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<Color> for [T; N] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
pub use Color::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Piece {
    #[default]
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Pawn,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            5 => Self::King,
            _ => unreachable!(),
        }
    }
}

impl<T, const N: usize> Index<Piece> for [T; N] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<Piece> for [T; N] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

pub use Piece::*;

use crate::search::INF;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ScoreType {
    #[default]
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone, Debug)]
pub struct AspirationWindow {
    pub lower: i32,
    pub upper: i32,
}

impl AspirationWindow {
    pub fn new(upper: i32, lower: i32) -> Self {
        Self { upper, lower }
    }
    pub fn new_around(mid: i32, width: i32) -> Self {
        Self {
            lower: mid.saturating_sub(width / 2).max(-INF),
            upper: mid.saturating_add(width / 2).min(INF),
        }
    }
    pub fn expand_down(&mut self, scale_percent: i32) {
        let mid = (self.upper + self.lower) / 2;
        let diff = (mid - self.lower).abs();
        self.lower = mid - diff.saturating_mul(scale_percent).div(100).max(-INF);
    }
    pub fn expand_up(&mut self, scale_percent: i32) {
        let mid = (self.upper + self.lower) / 2;
        let diff = (mid - self.upper).abs();
        self.upper = mid + diff.saturating_mul(scale_percent).div(100).max(-INF);
    }
    pub fn test(&self, score: i32) -> ScoreType {
        if score <= self.lower {
            ScoreType::UpperBound
        } else if score >= self.upper {
            ScoreType::LowerBound
        } else {
            ScoreType::Exact
        }
    }
}

impl Default for AspirationWindow {
    fn default() -> Self {
        Self {
            lower: -INF,
            upper: INF,
        }
    }
}
