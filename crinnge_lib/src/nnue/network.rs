use super::{Accumulator, EVAL_SCALE, HIDDEN_SIZE, QA, QB};

#[repr(C)]
pub struct Network {
    pub feature_weights: [Accumulator; 768],
    pub feature_bias: Accumulator,
    pub output_weights: [i16; HIDDEN_SIZE],
    pub output_bias: i16,
}

impl Network {
    pub fn evaluate(&self, acc: &Accumulator) -> i16 {
        let mut output = self.output_bias as i32;
        for (&n, &ow) in acc.vals.iter().zip(self.output_weights.iter()) {
            output += crelu(n) * i32::from(ow)
        }

        (output * EVAL_SCALE / (QA * QB)) as i16
    }
}

fn crelu(n: i16) -> i32 {
    i32::from(n).clamp(0, QA)
}
