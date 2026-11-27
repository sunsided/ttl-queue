//! [`TokioClock`]: a [`MonotonicClock`] over [`tokio::time::Instant`].

use timebridge::{ExactDuration, MonotonicClock};
use tokio::time::Instant;

/// A monotonic clock backed by [`tokio::time::Instant`].
///
/// Follows tokio's clock, so paused time (`tokio::time::pause`) and
/// `tokio::time::advance` drive expiry in tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TokioClock;

impl MonotonicClock for TokioClock {
    type Instant = Instant;

    fn read(&self) -> Instant {
        Instant::now()
    }

    fn duration_between(&self, start: Instant, end: Instant) -> ExactDuration {
        match end.checked_duration_since(start) {
            Some(forward) => exact(forward),
            None => exact(start.duration_since(end))
                .checked_neg()
                .expect("a positive ExactDuration always negates"),
        }
    }
}

/// Converts a clock interval into an [`ExactDuration`].
///
/// # Panics
///
/// Panics if the interval exceeds `i64::MAX` seconds, which no real clock reaches.
fn exact(duration: std::time::Duration) -> ExactDuration {
    duration
        .try_into()
        .expect("clock intervals never exceed i64::MAX seconds")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TtlQueue;
    use std::time::Duration;

    /// Expiry follows tokio's paused clock, not the wall clock.
    #[tokio::test(start_paused = true)]
    async fn follows_paused_time() {
        let mut queue = TtlQueue::with_clock(Duration::from_secs(10), TokioClock);
        queue.push_back(1);
        tokio::time::advance(Duration::from_secs(5)).await;
        queue.push_back(2);

        assert_eq!(queue.avg_delta(), Duration::from_secs(5));
        tokio::time::advance(Duration::from_secs(5)).await;
        assert_eq!(queue.refresh(), 1);
    }

    /// Readings in either order give signed durations of equal size.
    #[test]
    fn duration_between_is_signed() {
        let start = Instant::now();
        let end = start + Duration::from_secs(2);
        let forward = TokioClock.duration_between(start, end);

        assert_eq!(forward.as_nanos(), 2_000_000_000);
        assert_eq!(
            TokioClock.duration_between(end, start).as_nanos(),
            -2_000_000_000
        );
    }
}
