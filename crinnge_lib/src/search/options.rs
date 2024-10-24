use std::fmt::Display;

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
    pub nmp_min_depth: i32,
    pub nmp_r_const: i32,
    pub nmp_r_depth_divisor: i32,
    pub rfp_max_depth: i32,
    pub rfp_margin: i32,
    pub iir_min_depth: i32,
    pub iir_tt_depth_margin: i32,
    pub see_pruning_max_depth: i32,
    pub see_capture_margin: i32,
    pub see_quiet_margin: i32,
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
            nmp_min_depth: 1,
            nmp_r_const: 3,
            nmp_r_depth_divisor: 3,
            rfp_max_depth: 16,
            rfp_margin: 38,
            iir_min_depth: 4,
            iir_tt_depth_margin: 128, // high value effectively disables this feature
            see_pruning_max_depth: 10,
            see_capture_margin: -54,
            see_quiet_margin: -45,
        }
    }
}

impl Display for SearchOptions {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "option name Threads type spin default {} min 1 max 999", self.threads)?;
        writeln!(f, "option name Hash type spin default {} min 1 max 999", self.hash)?;
        #[cfg(feature = "tuning")] {
        use super::{INF, MAX_DEPTH};
        writeln!(f, "option name AspWindowInit type spin default {} min 1 max {}", self.asp_window_init, INF)?;
        writeln!(f, "option name AspWindowScalePercent type spin default {} min 101 max 999", self.asp_window_scale_percent)?;
        writeln!(f, "option name HardTimePercent type spin default {} min 1 max 100", self.hard_time_percent)?;
        writeln!(f, "option name SoftTimePercent type spin default {} min 1 max 100", self.soft_time_percent)?;
        writeln!(f, "option name IncPercent type spin default {} min 1 max 100", self.inc_percent)?;
        writeln!(f, "option name NmpMinDepth type spin default {} min 0 max {}", self.nmp_min_depth, MAX_DEPTH)?;
        writeln!(f, "option name NmpReductionConst type spin default {} min 0 max {}", self.nmp_r_const, MAX_DEPTH)?;
        writeln!(f, "option name NmpReductionDepthDivisor type spin default {} min 1 max {}", self.nmp_r_const, MAX_DEPTH)?;
        writeln!(f, "option name RfpMaxDepth type spin default {} min 1 max {}", self.rfp_max_depth, MAX_DEPTH)?;
        writeln!(f, "option name RfpMargin type spin default {} min 1 max {}", self.rfp_margin, INF)?;
        writeln!(f, "option name IirMinDepth type spin default {} min 1 max {}", self.iir_min_depth, MAX_DEPTH)?;
        writeln!(f, "option name IirTtDepthMargin type spin default {} min 1 max {}", self.iir_tt_depth_margin, MAX_DEPTH)?;
        writeln!(f, "option name SeePruningMaxDepth type spin default {} min 1 max {}", self.see_pruning_max_depth, MAX_DEPTH)?;
        writeln!(f, "option name SeeCaptureMargin type spin default {} min -100 max 100", self.see_capture_margin)?;
        writeln!(f, "option name SeeQuietMargin type spin default {} min -100 max 100", self.see_quiet_margin)?;
        }
        Ok(())
    }
}
