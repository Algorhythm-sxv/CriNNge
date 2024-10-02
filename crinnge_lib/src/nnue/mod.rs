const HIDDEN_SIZE: usize = 64;
const EVAL_SCALE: i32 = 400;
const QA: i32 = 255;
const QB: i32 = 64;

pub mod feature;
pub mod accumulator;
pub mod network;

pub use accumulator::*;
pub use network::*;

pub static NNUE: Network = unsafe { std::mem::transmute(*include_bytes!("crinnge_v1-10.bin")) };

