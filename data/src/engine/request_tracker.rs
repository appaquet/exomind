use exocore_common::time::Clock;
use std::time::{Duration, Instant};

///
/// Handles time tracking of synchronization request and response for timeouts, retries and backoff.
///
pub struct RequestTracker {
    clock: Clock,
    config: RequestTrackerConfig,

    last_request_send: Option<Instant>,
    last_response_receive: Option<Instant>,
    nb_response_failure: usize,

    // next can_send_request() will return true
    force_next_request: Option<bool>,
}

impl RequestTracker {
    pub fn new(config: RequestTrackerConfig) -> RequestTracker {
        RequestTracker::new_with_clock(Clock::new(), config)
    }

    pub fn new_with_clock(clock: Clock, config: RequestTrackerConfig) -> RequestTracker {
        RequestTracker {
            clock,
            config,

            last_request_send: None,
            last_response_receive: None,
            nb_response_failure: 0,

            force_next_request: None,
        }
    }

    pub fn set_last_send(&mut self, time: Instant) {
        self.last_request_send = Some(time);
    }

    pub fn set_last_responded(&mut self, time: Instant) {
        self.last_response_receive = Some(time);
        self.nb_response_failure = 0;
    }

    pub fn can_send_request(&mut self) -> bool {
        if let Some(_force) = self.force_next_request.take() {
            return true;
        }

        let next_request_interval = self.next_request_interval();
        let should_send_request = self.last_request_send.map_or(true, |send_time| {
            (self.clock.instant() - send_time) >= next_request_interval
        });

        if should_send_request {
            if self.last_request_send.is_some() && !self.has_responded_last_request() {
                self.nb_response_failure += 1;
            }

            true
        } else {
            false
        }
    }

    pub fn force_next_request(&mut self) {
        self.force_next_request = Some(true);
    }

    fn next_request_interval(&self) -> Duration {
        let interval =
            self.config.base_interval + self.config.base_interval * self.nb_response_failure as u32;
        interval.min(self.config.max_interval)
    }

    fn has_responded_last_request(&self) -> bool {
        match (self.last_request_send, self.last_response_receive) {
            (Some(send), Some(resp)) if resp > send => true,
            _ => false,
        }
    }
}

///
/// Configuration for RequestTracker
///
#[derive(Clone, Copy, Debug)]
pub struct RequestTrackerConfig {
    base_interval: Duration,
    max_interval: Duration,
}

impl Default for RequestTrackerConfig {
    fn default() -> Self {
        RequestTrackerConfig {
            base_interval: Duration::from_secs(5),
            max_interval: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_common::time::Clock;

    #[test]
    fn test_can_send_request_interval() {
        let mock_clock = Clock::new_mocked();
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock.clone(), RequestTrackerConfig::default());

        // should be able to do request right away
        assert!(tracker.can_send_request());

        // if we have sent, we shouldn't be able to re-do a query until timeout
        tracker.set_last_send(mock_clock.instant());
        assert!(!tracker.can_send_request());
        assert!(!tracker.has_responded_last_request());

        // after timeout, we should be able to make query, but then # failures is increased
        mock_clock.set_fixed_instant(Instant::now() + Duration::from_millis(5001));
        assert!(tracker.can_send_request());
        assert!(!tracker.has_responded_last_request());
        assert_eq!(tracker.nb_response_failure, 1);
    }

    #[test]
    fn test_force_request() {
        let mock_clock = Clock::new_mocked();
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock.clone(), RequestTrackerConfig::default());

        tracker.can_send_request();
        tracker.set_last_send(mock_clock.instant());

        assert!(!tracker.can_send_request());
        tracker.force_next_request();
        assert!(tracker.can_send_request());
    }

    #[test]
    fn test_can_send_request_timeout_backoff() {
        let mock_clock = Clock::new_fixed_mocked(Instant::now());
        let mut tracker =
            RequestTracker::new_with_clock(mock_clock.clone(), RequestTrackerConfig::default());

        tracker.can_send_request();
        tracker.set_last_send(mock_clock.instant());
        assert_eq!(tracker.next_request_interval(), Duration::from_secs(5));

        mock_clock.add_fixed_instant_duration(Duration::from_millis(5001));
        tracker.can_send_request();
        tracker.set_last_send(mock_clock.instant());
        assert_eq!(tracker.next_request_interval(), Duration::from_secs(10));

        for _i in 0..10 {
            mock_clock.add_fixed_instant_duration(
                tracker.next_request_interval() + Duration::from_millis(1),
            );
            tracker.can_send_request();
            tracker.set_last_send(mock_clock.instant());
        }

        // should be capped to maximum
        assert_eq!(tracker.next_request_interval(), Duration::from_secs(30));
    }

}
