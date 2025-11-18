#[derive(Clone, Copy, Debug, Default)]
pub struct BenchmarkMetrics {
    pub last_tick_duration_ms: f64,
    pub last_snapshot_duration_ms: f64,
}

impl BenchmarkMetrics {
    pub fn update_tick(&mut self, duration: f64) {
        if duration >= 0.0 {
            self.last_tick_duration_ms = duration;
        }
    }

    pub fn update_snapshot(&mut self, duration: f64) {
        if duration >= 0.0 {
            self.last_snapshot_duration_ms = duration;
        }
    }
}
