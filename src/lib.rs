//! # Timed Queue
//!
//! A queue that drops its content after a given amount of time.
//!
//! ## Crate Features
//!
//! * `vecdeque` - Uses a `VecDeque` as the underlying data structure. Enabled by default.
//! * `doublestack` - Uses two stacks (`Vec`) as the underlying data structure. Mutually exclusive with `vecdeque`.
//! * `tokio` - Uses [`tokio::time::Instant`] instead of [`std::time::Instant`].
//!
//! ## Example
//!
//! To implement an FPS counter, you could use the following technique:
//!
//! ```
//! # use std::thread;
//! # use std::time::Duration;
//! # use ttl_queue::TtlQueue;
//! let mut fps_counter = TtlQueue::new(Duration::from_secs_f64(1.0));
//!
//! for i in 0..100 {
//!     // Register a new frame and return the number of frames observed
//!     // within the last second.
//!     let fps = fps_counter.refresh_and_push_back(());
//!     debug_assert!(fps >= 1);
//!
//!     // Sleep 10 ms to achieve a ~100 Hz frequency.
//!     thread::sleep(Duration::from_millis(10));
//! }
//!
//! let fps = fps_counter.refresh();
//! debug_assert!(fps >= 95 && fps <= 105);
//! ```

use std::time::Duration;

#[cfg(not(feature = "tokio"))]
use std::time::Instant;

#[cfg(feature = "tokio")]
use tokio::time::Instant;

#[cfg(feature = "vecdeque")]
use std::collections::VecDeque;

/// A queue that drops its content after a given amount of time.
///
/// ## Example
///
/// To implement an FPS counter, you could use the following technique:
///
/// ```
/// # use std::thread;
/// # use std::time::Duration;
/// # use ttl_queue::TtlQueue;
/// let mut fps_counter = TtlQueue::new(Duration::from_secs_f64(1.0));
///
/// for i in 0..100 {
///     // Register a new frame and return the number of frames observed
///     // within the last second.
///     let fps = fps_counter.refresh_and_push_back(());
///     debug_assert!(fps >= 1);
///
///     // Sleep 10 ms to achieve a ~100 Hz frequency.
///     thread::sleep(Duration::from_millis(10));
/// }
///
/// let fps = fps_counter.refresh();
/// debug_assert!(fps >= 95 && fps <= 105);
/// ```
#[derive(Debug)]
pub struct TtlQueue<T> {
    ttl: Duration,
    #[cfg(feature = "doublestack")]
    stack_1: Vec<(Instant, T)>,
    #[cfg(feature = "doublestack")]
    stack_2: Vec<(Instant, T)>,
    #[cfg(feature = "vecdeque")]
    queue: VecDeque<(Instant, T)>,
}

impl<T> TtlQueue<T> {
    /// Creates an empty [`TtlQueue`] with default capacity.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            #[cfg(feature = "doublestack")]
            stack_1: Vec::new(),
            #[cfg(feature = "doublestack")]
            stack_2: Vec::new(),
            #[cfg(feature = "vecdeque")]
            queue: VecDeque::new(),
        }
    }

    /// Creates an empty [`TtlQueue`] for at least `capacity` elements.
    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self {
            ttl,
            #[cfg(feature = "doublestack")]
            stack_1: Vec::with_capacity(capacity),
            #[cfg(feature = "doublestack")]
            stack_2: Vec::with_capacity(capacity),
            #[cfg(feature = "vecdeque")]
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Pushes an element to the end of the queue.
    pub fn push_back(&mut self, element: T) {
        let entry = (Instant::now(), element);
        #[cfg(feature = "doublestack")]
        {
            self.stack_1.push(entry);
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.push_back(entry)
        }
    }

    /// Pushes an element to the end of the queue and returns the number of items
    /// currently in the queue. This operation is O(N) at worst.
    pub fn refresh_and_push_back(&mut self, element: T) -> usize {
        let count = self.refresh();
        self.push_back(element);
        count + 1
    }

    /// Gets the element from the front of the queue if it exists, as well as the
    /// time instant at which it was added.
    pub fn pop_front(&mut self) -> Option<(Instant, T)> {
        #[cfg(feature = "doublestack")]
        {
            self.ensure_stack_full(false);
            self.stack_2.pop()
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.pop_front()
        }
    }

    /// Similar to [`pop_front`](Self::pop_front) but without removing the element.
    pub fn peek_front(&mut self) -> Option<&(Instant, T)> {
        #[cfg(feature = "doublestack")]
        {
            self.ensure_stack_full(false);
            self.stack_2.first()
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.front()
        }
    }

    #[cfg(feature = "doublestack")]
    fn ensure_stack_full(&mut self, force: bool) {
        if self.stack_2.is_empty() || force {
            while let Some(item) = self.stack_1.pop() {
                self.stack_2.push(item);
            }
        }
    }

    /// Gets the number elements currently in the queue, including potentially expired elements.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    pub fn len(&self) -> usize {
        #[cfg(feature = "doublestack")]
        {
            self.stack_1.len() + self.stack_2.len()
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.len()
        }
    }

    /// Returns `true` if the queue is definitely empty or `false` if the queue is
    /// possibly empty.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    pub fn is_empty(&self) -> bool {
        #[cfg(feature = "doublestack")]
        {
            self.stack_1.is_empty() && self.stack_2.is_empty()
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.is_empty()
        }
    }

    /// Refreshes the queue and returns the number of currently contained elements.
    #[cfg(feature = "doublestack")]
    pub fn refresh(&mut self) -> usize {
        let now = Instant::now();

        while let Some((instant, _element)) = self.stack_2.first() {
            if (now - *instant) < self.ttl {
                break;
            }

            let _result = self.stack_2.pop();
            debug_assert!(_result.is_some());
        }

        if !self.stack_2.is_empty() {
            return self.len();
        }

        while let Some((instant, _element)) = self.stack_1.first() {
            if (now - *instant) < self.ttl {
                break;
            }

            let _result = self.stack_1.pop();
            debug_assert!(_result.is_some());
        }

        debug_assert_eq!(self.stack_1.len(), self.len());
        self.stack_1.len()
    }

    /// Refreshes the queue and returns the number of currently contained elements.
    #[cfg(feature = "vecdeque")]
    pub fn refresh(&mut self) -> usize {
        let now = Instant::now();

        while let Some((instant, _element)) = self.queue.front() {
            if (now - *instant) < self.ttl {
                break;
            }

            let _result = self.queue.pop_front();
            debug_assert!(_result.is_some());
        }

        self.queue.len()
    }

    /// Returns an iterator to the data.
    pub fn iter(&self) -> impl Iterator<Item = &(Instant, T)> {
        #[cfg(feature = "doublestack")]
        {
            return DoubleStackIterator::new(&self);
        }
        #[cfg(feature = "vecdeque")]
        {
            self.queue.iter()
        }
    }
}

impl<T> IntoIterator for TtlQueue<T> {
    type Item = (Instant, T);

    #[cfg(feature = "vecdeque")]
    type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

    #[cfg(feature = "doublestack")]
    type IntoIter = std::iter::Chain<
        std::iter::Rev<std::vec::IntoIter<Self::Item>>,
        std::vec::IntoIter<Self::Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        #[cfg(feature = "vecdeque")]
        {
            self.queue.into_iter()
        }
        #[cfg(feature = "doublestack")]
        {
            self.stack_2
                .into_iter()
                .rev()
                .chain(self.stack_1.into_iter())
        }
    }
}

#[cfg(feature = "doublestack")]
pub struct DoubleStackIterator<'a, T> {
    queue: &'a TtlQueue<T>,
    stage: DoubleStackIteratorStage<'a, T>,
}

#[cfg(feature = "doublestack")]
enum DoubleStackIteratorStage<'a, T> {
    First(std::iter::Rev<std::slice::Iter<'a, (Instant, T)>>),
    Second(std::slice::Iter<'a, (Instant, T)>),
    Done,
}

#[cfg(feature = "doublestack")]
impl<'a, T> Iterator for DoubleStackIteratorStage<'a, T> {
    type Item = &'a (Instant, T);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DoubleStackIteratorStage::First(iter) => iter.next(),
            DoubleStackIteratorStage::Second(iter) => iter.next(),
            DoubleStackIteratorStage::Done => None,
        }
    }
}

#[cfg(feature = "doublestack")]
impl<'a, T> DoubleStackIterator<'a, T> {
    pub fn new(queue: &'a TtlQueue<T>) -> Self {
        Self {
            queue,
            stage: DoubleStackIteratorStage::First(queue.stack_2.iter().rev()),
        }
    }
}

#[cfg(feature = "doublestack")]
impl<'a, T> Iterator for DoubleStackIterator<'a, T> {
    type Item = &'a (Instant, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(element) = self.stage.next() {
                return Some(element);
            }

            if matches!(self.stage, DoubleStackIteratorStage::First(..)) {
                self.stage = DoubleStackIteratorStage::Second(self.queue.stack_1.iter());
                continue;
            }

            debug_assert!(matches!(self.stage, DoubleStackIteratorStage::Second(..)));

            self.stage = DoubleStackIteratorStage::Done;
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_works() {
        let mut queue = TtlQueue::new(Duration::from_millis(50));
        queue.push_back(10);
        queue.push_back(20);
        queue.push_back(30);
        assert_eq!(queue.refresh(), 3);

        let value = queue.pop_front().unwrap();
        assert_eq!(value.1, 10);

        assert_eq!(queue.refresh(), 2);

        thread::sleep(Duration::from_millis(50));
        assert_eq!(queue.refresh(), 0);
    }

    #[test]
    fn iter_works() {
        let mut queue = TtlQueue::new(Duration::MAX);
        for i in 0..1000 {
            queue.push_back((i * 10) as usize);

            // Ensure data is both in stack 1 and stack 2
            #[cfg(feature = "doublestack")]
            {
                if i == 500 {
                    queue.ensure_stack_full(true);
                }
            }
        }

        for (i, (_instant, value)) in queue.iter().enumerate() {
            assert_eq!(*value, i * 10);
        }
    }

    #[test]
    fn into_iter_works() {
        let mut queue = TtlQueue::new(Duration::MAX);
        for i in 0..100 {
            queue.push_back((i * 10) as usize);

            // Ensure data is both in stack 1 and stack 2
            #[cfg(feature = "doublestack")]
            {
                if i == 50 {
                    queue.ensure_stack_full(true);
                }
            }
        }

        for (i, (_instant, value)) in queue.into_iter().enumerate() {
            assert_eq!(value, i * 10);
        }
    }
}
