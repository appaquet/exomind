use std::{sync::atomic::AtomicU64, time::Duration};

use wasm_timer::Instant;

pub struct AtomicInstant {
    reference: Instant,
    delta: AtomicU64,
}

impl AtomicInstant {
    pub fn new() -> Self {
        Self {
            reference: Instant::now(),
            delta: AtomicU64::new(0),
        }
    }

    pub fn get(&self) -> Instant {
        self.reference + Duration::from_nanos(self.delta.load(std::sync::atomic::Ordering::Relaxed))
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.get()
    }

    pub fn update(&self, now: Instant) {
        let delta = now - self.reference;
        self.delta.store(
            delta.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub fn update_now(&self) {
        self.update(Instant::now())
    }
}

impl Default for AtomicInstant {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Instant> for AtomicInstant {
    fn from(instant: Instant) -> Self {
        AtomicInstant {
            reference: instant,
            delta: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn test_atomic_instant() {
        let ai = AtomicInstant::new();

        sleep(Duration::from_millis(100));
        let elapsed = ai.elapsed();
        assert!(
            elapsed >= Duration::from_millis(100),
            "elapsed: {elapsed:?}"
        );

        ai.update_now();
        let elapsed = ai.elapsed();
        assert!(elapsed < Duration::from_millis(100), "elapsed: {elapsed:?}");
    }
}
