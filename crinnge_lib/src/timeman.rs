use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug)]
pub struct TimeOptions {
    /// the maximum percent of the total remaining time to use
    hard_time_percent: i64,
    /// the fraction of time to finish after the current depth
    soft_time_percent: i64,
    /// what percent of the increment to consider part of the time remaining
    inc_percent: i64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TimeData {
    pub stm_time: i64,
    pub ntm_time: i64,
    pub stm_inc: i64,
    pub ntm_inc: i64,
    pub movestogo: Option<i64>,
}

#[derive(Copy, Clone, Debug)]
pub struct TimeManager {
    start_time: Instant,
    time_data: TimeData,
    hard_time: Option<Duration>,
    soft_time: Option<Duration>,
    depth_limit: Option<i32>,
    node_limit: Option<u64>,
}

impl TimeManager {
    pub fn new(start_time: Instant) -> Self {
        Self {
            start_time,
            time_data: TimeData::default(),
            hard_time: None,
            soft_time: None,
            depth_limit: None,
            node_limit: None,
        }
    }

    pub fn time_limited(mut self, data: TimeData, options: TimeOptions) -> Self {
        self.hard_time = Some(Duration::from_millis(
            ((data.stm_time * options.hard_time_percent + data.stm_inc * options.inc_percent) / 100)
                .max(data.stm_time / 10) // never use more than 90% of clock time, guard for low-time high-increment situations
                .max(0) as u64, // round negative times up to zero
        ));

        self.soft_time = Some(Duration::from_millis(
            ((data.stm_time * options.soft_time_percent + data.stm_inc * options.inc_percent) / 100)
                .max(0) as u64,
        ));

        self
    }

    pub fn fixed_time_millis(mut self, millis: i64) -> Self {
        self.hard_time = Some(Duration::from_millis(millis as u64));
        self.soft_time = self.hard_time;

        self
    }

    pub fn fixed_depth(mut self, depth: i32) -> Self {
        self.depth_limit = Some(depth);

        self
    }

    pub fn fixed_nodes(mut self, nodes: u64) -> Self {
        self.node_limit = Some(nodes);

        self
    }

    pub fn hard_time(&self) -> Option<Duration> {
        self.hard_time
    }

    pub fn soft_time(&self) -> Option<Duration> {
        self.soft_time
    }

    pub fn depth_limit(&self) -> Option<i32> {
        self.depth_limit
    }

    pub fn node_limit(&self) -> Option<u64> {
        self.node_limit
    }
}
