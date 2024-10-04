use std::ops::{Deref, DerefMut};

use super::{EVAL_SCALE, HIDDEN_SIZE, QA, QB};

#[repr(C)]
pub struct Network {
    pub feature_weights: [Aligned; 768],
    pub feature_bias: Aligned,
    pub output_weights: Aligned,
    pub output_bias: i16,
}

impl Network {
    pub fn evaluate(&self, acc: &Aligned) -> i32 {
        let mut output = self.output_bias as i32;
        for (&n, &ow) in acc.iter().zip(self.output_weights.iter()) {
            output += crelu(n) * i32::from(ow)
        }

        output * EVAL_SCALE / (QA * QB)
    }
}

fn crelu(n: i16) -> i32 {
    i32::from(n).clamp(0, QA)
}

#[repr(align(64))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Aligned([i16; HIDDEN_SIZE]);

impl Deref for Aligned {
    type Target = [i16; HIDDEN_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
