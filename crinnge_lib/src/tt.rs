use std::{
    mem::size_of,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{moves::Move, search::MIN_TB_WIN_SCORE, types::*};

#[derive(Copy, Clone, Debug, Default)]
pub struct TTEntryInfo(pub u8);

impl TTEntryInfo {
    pub fn new(score_type: ScoreType) -> Self {
        Self(score_type as u8 & 0b11)
    }
    pub fn score_type(&self) -> ScoreType {
        match self.0 & 0b11 {
            0 => ScoreType::Exact,
            1 => ScoreType::LowerBound,
            _ => ScoreType::UpperBound,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TTScore(i16);

impl TTScore {
    pub fn new(score: i32, ply: usize) -> Self {
        let tt_score = if score >= MIN_TB_WIN_SCORE {
            score + ply as i32
        } else if score <= -MIN_TB_WIN_SCORE {
            score - ply as i32
        } else {
            score
        };
        Self(tt_score as i16)
    }

    fn get(&self, ply: usize) -> i32 {
        let score = if self.0 >= MIN_TB_WIN_SCORE as i16 {
            self.0 - ply as i16
        } else if self.0 <= -MIN_TB_WIN_SCORE as i16 {
            self.0 + ply as i16
        } else {
            self.0
        };
        score as i32
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TTEntry {
    pub key: u16,
    pub best_move: Move,
    pub score: TTScore,
    pub depth: u8,
    pub info: TTEntryInfo,
}

const _TT_ENTRY_SIZE: () = assert!(size_of::<TTEntry>() <= 8, "TT entry does not fit in u64");

impl TTEntry {
    pub fn pack(&self) -> u64 {
        // SAFETY: u64 has no invalid bit patterns and TTEntry is the same size
        unsafe { std::mem::transmute(*self) }
    }
}

impl From<u64> for TTEntry {
    fn from(value: u64) -> Self {
        // SAFETY: TTEntry has no invalid bit patterns and is the same size as a u64
        unsafe { std::mem::transmute(value) }
    }
}

pub struct TT {
    entries: Vec<AtomicU64>,
}

impl TT {
    pub fn new(size_mb: usize) -> Self {
        let mut entries = vec![];
        for _ in 0..(size_mb * 1024 * 1024 / 8) {
            entries.push(AtomicU64::new(0));
        }
        Self { entries }
    }

    pub fn resize(&mut self, size_mb: usize) {
        self.entries
            .resize_with(size_mb * 1024 * 1024 / 8, || AtomicU64::new(0));
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = AtomicU64::new(0);
        }
    }

    pub fn slice(&self) -> TTSlice {
        TTSlice {
            entries: &self.entries[..],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TTSlice<'a> {
    entries: &'a [AtomicU64],
}

impl<'a> TTSlice<'a> {
    pub fn get(&self, key: u64) -> Option<TTEntry> {
        let index = self.key_to_index(key);
        let entry = TTEntry::from(self.entries[index].load(Ordering::Relaxed));
        if entry.key == key as u16 {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(
        &self,
        key: u64,
        score: i32,
        score_type: ScoreType,
        best_move: Move,
        depth: i32,
        ply: usize,
    ) {
        let entry = TTEntry {
            key: key as u16,
            best_move,
            score: TTScore::new(score, ply),
            depth: depth as u8,
            info: TTEntryInfo::new(score_type),
        };
        let index = self.key_to_index(key);
        self.entries[index].store(entry.pack(), Ordering::Relaxed);
    }

    fn key_to_index(&self, key: u64) -> usize {
        ((u128::from(key) * self.entries.len() as u128) >> 64) as usize
    }
}
