use std::time::{Duration, Instant};

use chrono::{DateTime, Local};

pub struct Booking {
    pub user: String,
    pub creation_datetime: DateTime<Local>,
    pub creation_instant: Instant,
    pub currently_running_since: Option<Instant>,
    pub runtime_accumulator: Duration
}

impl Booking {
    pub fn new(user: String) -> Self {
        Self {
            user,
            creation_datetime: Local::now(),
            creation_instant: Instant::now(),
            currently_running_since: None,
            runtime_accumulator: Duration::ZERO
        }
    }

    pub fn track(&mut self, power: bool) -> bool {
        if self.is_running() == power {
            return false;
        }

        if power {
            self.currently_running_since = Some(Instant::now());
        } else {
            self.runtime_accumulator += self
                .currently_running_since
                .take()
                .unwrap() //SAFETY: guaranteed by .is_running()
                .elapsed();
        }

        true
    }

    pub const fn is_running(&self) -> bool {
        self.currently_running_since.is_some()
    }

    pub fn total_runtime(&self) -> Duration {
        let mut total = self.runtime_accumulator;

        if let Some(startup) = self.currently_running_since {
            total += startup.elapsed();
        }

        total
    }
}