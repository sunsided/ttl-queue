//! Kani proof harnesses.
//!
//! Kani cannot model `clock_gettime`, so the harnesses drive the queue with
//! [`StepClock`], a clock whose reading only changes when the harness says so.
//!
//! Run with `cargo kani`.

use crate::TtlQueue;
use std::cell::Cell;
use std::time::Duration;
use timebridge::{ExactDuration, MonotonicClock};

/// A manual clock: the reading is a plain offset that the harness advances.
#[derive(Debug, Default)]
struct StepClock {
    /// The current reading.
    now: Cell<Duration>,
}

impl StepClock {
    /// Moves the clock forward by a nondeterministic number of milliseconds (0..=255).
    fn advance_any(&self) {
        let step = Duration::from_millis(u64::from(kani::any::<u8>()));
        self.now.set(self.now.get() + step);
    }
}

impl MonotonicClock for StepClock {
    type Instant = Duration;

    fn read(&self) -> Duration {
        self.now.get()
    }

    fn duration_between(&self, start: Duration, end: Duration) -> ExactDuration {
        let forward = ExactDuration::try_from(end.saturating_sub(start)).expect("small offsets");
        let backward = ExactDuration::try_from(start.saturating_sub(end)).expect("small offsets");
        forward.checked_sub(backward).expect("small offsets")
    }
}

/// Maximum number of elements pushed per harness; keeps the state space small.
const MAX_ELEMENTS: usize = 3;

/// Pushes up to [`MAX_ELEMENTS`] elements at nondecreasing instants.
///
/// Element `i` has value `i`. Returns the push instants and how many were pushed.
fn push_any(
    queue: &mut TtlQueue<usize, &StepClock>,
    clock: &StepClock,
) -> ([Duration; MAX_ELEMENTS], usize) {
    let mut instants = [Duration::ZERO; MAX_ELEMENTS];
    let count: usize = kani::any_where(|n| *n <= MAX_ELEMENTS);

    for (i, slot) in instants.iter_mut().enumerate().take(count) {
        clock.advance_any();
        *slot = clock.read();
        queue.push_back(i);
    }

    (instants, count)
}

/// `refresh` removes exactly the expired prefix and keeps the rest in order.
#[kani::proof]
#[kani::unwind(5)]
fn refresh_drops_exactly_expired() {
    let clock = StepClock::default();
    let ttl = Duration::from_millis(u64::from(kani::any::<u8>()));
    let mut queue = TtlQueue::with_clock(ttl, &clock);
    let (instants, count) = push_any(&mut queue, &clock);
    clock.advance_any();
    let now = clock.read();

    let expired = instants[..count]
        .iter()
        .filter(|t| now - **t >= ttl)
        .count();
    let remaining = queue.refresh();

    assert_eq!(remaining, count - expired);
    for (offset, (instant, value)) in queue.iter().enumerate() {
        assert_eq!(*value, expired + offset);
        assert!(now - *instant < ttl);
    }
}

/// `avg_delta` is the span between the oldest and newest element, divided by the gap count.
#[kani::proof]
#[kani::unwind(5)]
fn avg_delta_is_span_over_gaps() {
    let clock = StepClock::default();
    let mut queue = TtlQueue::with_clock(Duration::MAX, &clock);
    let (instants, count) = push_any(&mut queue, &clock);
    let avg = queue.avg_delta();

    if count < 2 {
        assert_eq!(avg, Duration::ZERO);
        return;
    }

    let span = instants[count - 1] - instants[0];
    let gaps = u32::try_from(count - 1).expect("at most MAX_ELEMENTS gaps");
    assert!(avg * gaps <= span);
    assert!(span < (avg + Duration::from_nanos(1)) * gaps);
}
