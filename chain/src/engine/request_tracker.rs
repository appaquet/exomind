use exocore_core::time::{Clock, Instant};
use exocore_core::utils::backoff::{BackoffCalculator, BackoffConfig};
use std::time::Duration;

/// Handles time tracking of synchronization request and response for timeouts,
/// retries and backoff.
pub struct RequestTracker {
    clock: Clock,
    backoff_calculator: BackoffCalculator,

    last_request_send: Option<Instant>,
    last_response_receive: Option<Instant>,

    // next can_send_request() will return true
    force_next_request: Option<bool>,
}

impl RequestTracker {
    pub fn new_with_clock(clock: Clock, config: RequestTrackerConfig) -> RequestTracker {
        let backoff_calculator = BackoffCalculator::new(
            clock.clone(),
            BackoffConfig {
                normal_constant: config.min_interval,
                failure_constant: config.min_interval,
                failure_exp_base: 2.0,
                failure_exp_multiplier: Duration::from_secs(5),
                failure_maximum: config.max_interval,
            },
        );

        RequestTracker {
            clock,
            backoff_calculator,

            last_request_send: None,
            last_response_receive: None,

            force_next_request: None,
        }
    }

    pub fn set_last_send_now(&mut self) {
        self.last_request_send = Some(self.clock.instant());
    }

    pub fn set_last_responded_now(&mut self) {
        self.last_response_receive = Some(self.clock.instant());
        self.backoff_calculator.reset();
    }

    pub fn can_send_request(&mut self) -> bool {
        if let Some(_force) = self.force_next_request.take() {
            return true;
        }

        let should_send_request = self.last_request_send.map_or(true, |send_time| {
            (self.clock.instant() - send_time) >= self.backoff_calculator.backoff_duration()
        });

        if should_send_request {
            if self.last_request_send.is_some() && !self.has_responded_last_request() {
                self.backoff_calculator.increment_failure();
            }

            true
        } else {
            false
        }
    }

    pub fn force_next_request(&mut self) {
        self.force_next_request = Some(true);
    }

    pub fn reset(&mut self) {
        self.last_request_send = None;
        self.last_response_receive = None;
        self.force_next_request = None;
    }

    fn has_responded_last_request(&self) -> bool {
        matches!((self.last_request_send, self.last_response_receive), (Some(send), Some(resp)) if resp > send)
    }

    pub fn response_failure_count(&self) -> usize {
        self.backoff_calculator.consecutive_failures_count() as usize
    }

    #[cfg(test)]
    pub fn set_response_failure_count(&mut self, count: usize) {
        for _ in 0..count {
            self.backoff_calculator.increment_failure();
        }
    }
}

/// Configuration for RequestTracker
#[derive(Clone, Copy, Debug)]
pub struct RequestTrackerConfig {
    pub min_interval: Duration,
    pub max_interval: Duration,
}

impl Default for RequestTrackerConfig {
    fn default() -> Self {
        RequestTrackerConfig {
            min_interval: Duration::from_secs(5),
            max_interval: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_core::time::Clock;

    #[test]
    fn test_can_send_request_interval() {
        let mock_clock = Clock::new_mocked();
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock.clone(), RequestTrackerConfig::default());

        // should be able to do request right away
        assert!(tracker.can_send_request());

        // if we have sent, we shouldn't be able to re-do a query until timeout
        tracker.set_last_send_now();
        assert!(!tracker.can_send_request());
        assert!(!tracker.has_responded_last_request());

        // after timeout, we should be able to make query, but then # failures is
        // increased
        mock_clock.set_fixed_instant(Instant::now() + Duration::from_millis(5001));
        assert!(tracker.can_send_request());
        assert!(!tracker.has_responded_last_request());
        assert_eq!(tracker.response_failure_count(), 1);
    }

    #[test]
    fn test_force_request() {
        let mock_clock = Clock::new_mocked();
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock, RequestTrackerConfig::default());

        tracker.can_send_request();
        tracker.set_last_send_now();

        assert!(!tracker.can_send_request());
        tracker.force_next_request();
        assert!(tracker.can_send_request());
    }

    #[test]
    fn test_reset() {
        let mock_clock = Clock::new_mocked();
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock, RequestTrackerConfig::default());

        tracker.can_send_request();
        tracker.set_last_send_now();

        assert!(!tracker.can_send_request());
        tracker.reset();
        assert!(tracker.can_send_request());
    }
}
