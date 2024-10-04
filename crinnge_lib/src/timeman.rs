use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug)]
pub struct TimeOptions {
    /// the maximum percent of the total remaining time to use
    pub hard_time_percent: i64,
    /// the fraction of time to finish after the current depth
    pub soft_time_percent: i64,
    /// what percent of the increment to consider part of the time remaining
    pub inc_percent: i64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TimeData {
    pub stm_time: i64,
    pub ntm_time: i64,
    pub stm_inc: i64,
    pub ntm_inc: i64,
    pub movestogo: Option<usize>,
}

#[derive(Copy, Clone, Debug)]
pub struct TimeManager {
    start_time: Instant,
    time_data: TimeData,
    hard_time: Option<Duration>,
    soft_time: Option<Duration>,
    depth_limit: Option<usize>,
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
        self.time_data = data;
        self.hard_time = Some(Duration::from_millis(
            ((data.stm_time * options.hard_time_percent + data.stm_inc * options.inc_percent) / 100)
                .max(data.stm_time / 10) // never use more than 90% of clock time, guard for low-time high-increment situations
                .max(0) as u64, // round negative times up to zero
        ));

        self.soft_time = Some(Duration::from_millis(
            ((data.stm_time * options.soft_time_percent + data.stm_inc * options.inc_percent) / 100)
                .max(0) as u64,
        ));

        if let Some(movestogo) = data.movestogo {
            self.soft_time = Some(Duration::from_millis(
                (data.stm_time / movestogo as i64).max(0) as u64,
            ));
        }

        self
    }

    pub fn fixed_time_millis(mut self, millis: Option<i64>) -> Self {
        if let Some(millis) = millis {
            self.hard_time = Some(Duration::from_millis(millis as u64));
            self.soft_time = None;
        }

        self
    }

    pub fn infinite(mut self, infinite: bool) -> Self {
        if infinite {
            self.hard_time = None;
            self.soft_time = None;
        }

        self
    }

    pub fn fixed_depth(mut self, depth: Option<usize>) -> Self {
        self.depth_limit = depth;
        if depth.is_some() {
            self.soft_time = None;
            self.hard_time = None;
        }

        self
    }

    pub fn fixed_nodes(mut self, nodes: Option<u64>) -> Self {
        self.node_limit = nodes;
        if nodes.is_some() {
            self.soft_time = None;
            self.hard_time = None;
        }

        self
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn depth_limit_reached(&self, depth: i32) -> bool {
        if let Some(limit) = self.depth_limit {
            depth >= limit as i32
        } else {
            false
        }
    }

    pub fn node_limit_reached(&self, nodes: u64) -> bool {
        if let Some(limit) = self.node_limit {
            nodes >= limit
        } else {
            false
        }
    }

    pub fn soft_time_limit_reached(&self) -> bool {
        if let Some(limit) = self.soft_time {
            self.elapsed() >= limit
        } else {
            false
        }
    }

    pub fn hard_time_limit_reached(&self) -> bool {
        if let Some(limit) = self.hard_time {
            self.elapsed() >= limit
        } else {
            false
        }
    }
}
