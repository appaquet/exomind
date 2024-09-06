use crate::time::{Clock, Duration, Instant};

#[derive(Clone, Copy)]
pub struct BackoffConfig {
    pub normal_constant: Duration,
    pub failure_constant: Duration,
    pub failure_exp_base: f32,
    pub failure_exp_multiplier: Duration,
    pub failure_maximum: Duration,
}

pub struct BackoffCalculator {
    clock: Clock,
    config: BackoffConfig,
    consecutive_failures_count: u32,
    next_execution_allow: Option<Instant>,
}

impl BackoffCalculator {
    pub fn new(clock: Clock, config: BackoffConfig) -> BackoffCalculator {
        BackoffCalculator {
            clock,
            config,
            consecutive_failures_count: 0,
            next_execution_allow: None,
        }
    }

    pub fn can_execute_now(&self) -> bool {
        if self.consecutive_failures_count == 0 {
            return true;
        }

        self.next_execution_time() <= self.clock.instant()
    }

    pub fn next_execution_time(&self) -> Instant {
        self.next_execution_allow
            .unwrap_or_else(|| self.clock.instant())
    }

    pub fn increment_failure(&mut self) {
        self.consecutive_failures_count += 1;
        self.next_execution_allow = Some(self.clock.instant() + self.backoff_duration());
    }

    pub fn consecutive_failures_count(&self) -> u32 {
        self.consecutive_failures_count
    }

    pub fn reset(&mut self) {
        self.consecutive_failures_count = 0;
        self.next_execution_allow = None;
    }

    pub fn backoff_duration(&self) -> Duration {
        if self.consecutive_failures_count == 0 {
            self.config.normal_constant
        } else {
            let multiplier =
                self.config
                    .failure_exp_base
                    .powi((self.consecutive_failures_count - 1) as i32) as u32;

            let duration =
                self.config.failure_constant + self.config.failure_exp_multiplier * multiplier;

            duration.min(self.config.failure_maximum)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_case() {
        let clock = Clock::new();
        let mut exp = BackoffCalculator::new(
            clock,
            BackoffConfig {
                normal_constant: Duration::new(0, 0),
                failure_constant: Duration::from_millis(5),
                failure_exp_base: 2.0,
                failure_exp_multiplier: Duration::from_millis(10),
                failure_maximum: Duration::from_millis(50),
            },
        );

        assert!(exp.can_execute_now());

        exp.increment_failure();
        assert_eq!(exp.backoff_duration().as_millis(), 15); // 5 + 2**0 * 10.0

        exp.increment_failure();
        assert_eq!(exp.backoff_duration().as_millis(), 25);

        exp.increment_failure();
        assert_eq!(exp.backoff_duration().as_millis(), 45);

        exp.increment_failure();
        assert_eq!(exp.backoff_duration().as_millis(), 50); // max

        assert!(!exp.can_execute_now());

        exp.reset();

        assert!(exp.can_execute_now());
    }
}
