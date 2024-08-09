use std::time::{Duration, Instant};

pub struct Timer {
    count_down: Duration,
    time: Duration,
}

impl Timer {
    pub fn new(time: Duration) -> Self {
        Timer {
            count_down: Duration::new(0, 0),
            time,
        }
    }

    pub fn from_secs_f64(time: f64) -> Self {
        Self::new(Duration::from_secs_f64(time))
    }
    pub fn from_secs_f32(time: f32) -> Self {
        Self::new(Duration::from_secs_f32(time))
    }

    pub fn tick(&mut self, to_tick: Duration) -> bool {
        self.count_down += to_tick;

        self.count_down >= self.time
    }
}

pub struct InstantTimer {
    start: Instant,
    time: Duration,
}

impl InstantTimer {
    pub fn new(time: Duration) -> Self {
        Self {
            start: Instant::now(),
            time,
        }
    }

    pub fn from_secs_f64(time: f64) -> Self {
        Self::new(Duration::from_secs_f64(time))
    }
    pub fn from_secs_f32(time: f32) -> Self {
        Self::new(Duration::from_secs_f32(time))
    }

    #[inline]
    pub fn tick(&self) -> bool {
        self.start.elapsed() >= self.time
    }

    pub fn tick_reset(&mut self) -> bool {
        if self.tick() {
            self.reset();
            return true;
        }
        false
    }

    #[inline]
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}
