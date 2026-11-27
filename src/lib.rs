#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::collections::vec_deque;
use std::time::Duration;

pub use timebridge::MonotonicClock;
pub use timebridge_clock::SystemClock;

#[cfg(feature = "tokio")]
mod tokio_clock;

#[cfg(feature = "tokio")]
pub use tokio_clock::TokioClock;

#[cfg(kani)]
mod kani_proofs;

#[cfg(test)]
mod proptests;

/// A queue that drops its content after a given amount of time.
///
/// Elements are stored in insertion order together with the clock reading at which
/// they were pushed. Expired elements are only removed when [`refresh`](Self::refresh)
/// (or [`refresh_and_push_back`](Self::refresh_and_push_back)) is called.
///
/// Time comes from a [`MonotonicClock`]; [`SystemClock`] (backed by
/// [`std::time::Instant`]) is the default. Use [`with_clock`](Self::with_clock) to
/// supply another one, such as `TokioClock` (feature `tokio`) or a manual test clock.
///
/// See the [crate-level documentation](crate) for an example.
#[derive(Debug)]
pub struct TtlQueue<T, C: MonotonicClock = SystemClock> {
    /// How long an element stays in the queue after it was pushed.
    ttl: Duration,
    /// The time source for insertion instants and expiry checks.
    clock: C,
    /// The elements with their insertion instants, oldest at the front.
    queue: VecDeque<(C::Instant, T)>,
}

impl<T> TtlQueue<T> {
    /// Creates an empty [`TtlQueue`] on the [`SystemClock`] with default capacity.
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self::with_clock(ttl, SystemClock)
    }

    /// Creates an empty [`TtlQueue`] on the [`SystemClock`] for at least `capacity` elements.
    #[must_use]
    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self {
            ttl,
            clock: SystemClock,
            queue: VecDeque::with_capacity(capacity),
        }
    }
}

impl<T, C: MonotonicClock> TtlQueue<T, C> {
    /// Creates an empty [`TtlQueue`] that reads time from `clock`.
    #[must_use]
    pub fn with_clock(ttl: Duration, clock: C) -> Self {
        Self {
            ttl,
            clock,
            queue: VecDeque::new(),
        }
    }

    /// Pushes an element to the end of the queue.
    pub fn push_back(&mut self, element: T) {
        let now = self.clock.read();
        self.queue.push_back((now, element));
    }

    /// Refreshes the queue, pushes an element to the end and returns the number of
    /// items currently in the queue. This operation is O(N) at worst.
    pub fn refresh_and_push_back(&mut self, element: T) -> usize {
        let count = self.refresh();
        self.push_back(element);

        count + 1
    }

    /// Removes the element from the front of the queue if it exists, and returns it
    /// together with the clock reading at which it was added.
    pub fn pop_front(&mut self) -> Option<(C::Instant, T)> {
        self.queue.pop_front()
    }

    /// Similar to [`pop_front`](Self::pop_front) but without removing the element.
    #[must_use]
    pub fn peek_front(&self) -> Option<&(C::Instant, T)> {
        self.queue.front()
    }

    /// Gets the number elements currently in the queue, including potentially expired elements.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if the queue holds no elements, not even expired ones.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Drops all expired elements and returns the number of remaining elements.
    ///
    /// An element pushed at `t` is expired once `now - t >= ttl`.
    pub fn refresh(&mut self) -> usize {
        let now = self.clock.read();

        while self
            .queue
            .front()
            .is_some_and(|(instant, _)| self.span(*instant, now) >= self.ttl)
        {
            self.queue.pop_front();
        }

        self.queue.len()
    }

    /// Returns an iterator over the elements, oldest first, including potentially
    /// expired elements.
    #[must_use]
    pub fn iter(&self) -> vec_deque::Iter<'_, (C::Instant, T)> {
        self.queue.iter()
    }

    /// Returns the average duration between two consecutive elements.
    ///
    /// Includes potentially expired elements; call [`refresh`](Self::refresh) first
    /// for an accurate value. Returns [`Duration::ZERO`] for fewer than two elements.
    #[must_use]
    pub fn avg_delta(&self) -> Duration {
        let (Some(first), Some(last)) = (self.queue.front(), self.queue.back()) else {
            return Duration::ZERO;
        };

        let gaps = self.queue.len() - 1;
        if gaps == 0 {
            return Duration::ZERO;
        }

        // shortcut: saturates above u32::MAX gaps, upgrade if queues ever get that large.
        let gaps = u32::try_from(gaps).unwrap_or(u32::MAX);
        self.span(first.0, last.0) / gaps
    }

    /// Returns the time from `start` to `end`, or zero if `end` is earlier.
    fn span(&self, start: C::Instant, end: C::Instant) -> Duration {
        Duration::try_from(self.clock.duration_between(start, end)).unwrap_or(Duration::ZERO)
    }
}

impl<T, C: MonotonicClock> IntoIterator for TtlQueue<T, C> {
    type Item = (C::Instant, T);
    type IntoIter = vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}

impl<'a, T, C: MonotonicClock> IntoIterator for &'a TtlQueue<T, C> {
    type Item = &'a (C::Instant, T);
    type IntoIter = vec_deque::Iter<'a, (C::Instant, T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use timebridge::Timestamp;
    use timebridge_clock::ManualClock;

    /// A manual clock starting at the Unix epoch.
    fn manual_clock() -> ManualClock {
        ManualClock::new(Timestamp::UNIX_EPOCH)
    }

    /// Pushing, popping and expiry work in insertion order.
    #[test]
    fn it_works() {
        let clock = manual_clock();
        let mut queue = TtlQueue::with_clock(Duration::from_millis(50), &clock);
        queue.push_back(10);
        queue.push_back(20);
        queue.push_back(30);
        assert_eq!(queue.refresh(), 3);

        let value = queue.pop_front().unwrap();
        assert_eq!(value.1, 10);
        assert_eq!(queue.refresh(), 2);

        clock.advance(Duration::from_millis(50));
        assert_eq!(queue.refresh(), 0);
    }

    /// The default [`SystemClock`] queue keeps fresh elements.
    #[test]
    fn system_clock_works() {
        let mut queue = TtlQueue::new(Duration::MAX);
        queue.push_back(1);
        assert_eq!(queue.refresh_and_push_back(2), 2);
    }

    /// `peek_front` returns the same element `pop_front` removes next.
    #[test]
    fn peek_matches_pop() {
        let mut queue = TtlQueue::with_clock(Duration::MAX, manual_clock());
        queue.push_back(1);
        queue.push_back(2);
        queue.push_back(3);

        let peeked = queue.peek_front().unwrap().1;
        assert_eq!(peeked, 1);
        assert_eq!(queue.pop_front().unwrap().1, peeked);
    }

    /// `refresh` drops only expired elements and keeps fresh ones.
    #[test]
    fn refresh_drops_only_expired() {
        let clock = manual_clock();
        let mut queue = TtlQueue::with_clock(Duration::from_millis(50), &clock);
        queue.push_back(1);
        clock.advance(Duration::from_millis(60));
        queue.push_back(2);

        assert_eq!(queue.refresh(), 1);
        assert_eq!(queue.pop_front().unwrap().1, 2);
    }

    /// `iter` yields elements oldest first.
    #[test]
    fn iter_works() {
        let mut queue = TtlQueue::with_clock(Duration::MAX, manual_clock());
        for i in 0..1000_usize {
            queue.push_back(i * 10);
        }

        let values: Vec<usize> = queue.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, (0..1000).map(|i| i * 10).collect::<Vec<_>>());
    }

    /// Iterating `&TtlQueue` matches [`TtlQueue::iter`].
    #[test]
    fn ref_into_iter_works() {
        let mut queue = TtlQueue::with_clock(Duration::MAX, manual_clock());
        queue.push_back(1);
        queue.push_back(2);

        let values: Vec<i32> = (&queue).into_iter().map(|(_, v)| *v).collect();
        assert_eq!(values, [1, 2]);
    }

    /// `into_iter` yields elements oldest first.
    #[test]
    fn into_iter_works() {
        let mut queue = TtlQueue::with_clock(Duration::MAX, manual_clock());
        for i in 0..100_usize {
            queue.push_back(i * 10);
        }

        let values: Vec<usize> = queue.into_iter().map(|(_, v)| v).collect();
        assert_eq!(values, (0..100).map(|i| i * 10).collect::<Vec<_>>());
    }

    /// `avg_delta` averages the gaps between evenly spaced instants.
    #[test]
    fn avg_duration_works() {
        let clock = manual_clock();
        let mut queue = TtlQueue::with_clock(Duration::MAX, &clock);
        for _ in 0..10 {
            queue.push_back(());
            clock.advance(Duration::from_secs(1));
        }

        assert_eq!(queue.avg_delta(), Duration::from_secs(1));
    }

    /// `avg_delta` averages uneven gaps.
    #[test]
    fn avg_duration_uneven_works() {
        let clock = manual_clock();
        let mut queue = TtlQueue::with_clock(Duration::MAX, &clock);
        for gap in [0, 1, 4] {
            clock.advance(Duration::from_secs(gap));
            queue.push_back(());
        }

        assert_eq!(queue.avg_delta(), Duration::from_millis(2500));
    }

    /// `avg_delta` of an empty queue is zero.
    #[test]
    fn avg_duration_with_zero_inputs_works() {
        let queue = TtlQueue::<()>::new(Duration::MAX);
        assert_eq!(queue.avg_delta(), Duration::ZERO);
    }

    /// `avg_delta` of a single element is zero.
    #[test]
    fn avg_duration_with_one_inputs_works() {
        let mut queue = TtlQueue::new(Duration::MAX);
        queue.push_back(());
        assert_eq!(queue.avg_delta(), Duration::ZERO);
    }

    /// `refresh_and_push_back` counts the new element plus all unexpired ones.
    #[test]
    fn refresh_and_push_back_counts() {
        let mut queue = TtlQueue::with_clock(Duration::MAX, manual_clock());
        for expected in 1..=50 {
            assert_eq!(queue.refresh_and_push_back(()), expected);
        }
    }
}
