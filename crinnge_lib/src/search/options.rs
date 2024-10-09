use crate::timeman::TimeOptions;

#[derive(Copy, Clone, Debug)]
pub struct SearchOptions {
    pub threads: usize,
    pub hash: usize,
    pub asp_window_init: i32,
    pub asp_window_scale_percent: i32,
    pub hard_time_percent: i64,
    pub soft_time_percent: i64,
    pub inc_percent: i64,
    pub nmp_depth: i32,
    pub nmp_r_const: i32,
    pub nmp_r_depth_divisor: i32,
}

impl SearchOptions {
    pub fn time_options(&self) -> TimeOptions {
        TimeOptions {
            hard_time_percent: self.hard_time_percent,
            soft_time_percent: self.soft_time_percent,
            inc_percent: self.inc_percent,
        }
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            threads: 1,
            hash: 8,
            asp_window_init: 40,
            asp_window_scale_percent: 200,
            hard_time_percent: 50,
            soft_time_percent: 5,
            inc_percent: 50,
            nmp_depth: 1,
            nmp_r_const: 3,
            nmp_r_depth_divisor: 3,
        }
    }
}
