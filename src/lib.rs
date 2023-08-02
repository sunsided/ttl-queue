use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A queue that drops its content after a given amount of time.
#[derive(Debug)]
pub struct TimedQueue<T> {
    ttl: Duration,
    stack_1: Vec<(Instant, T)>,
    stack_2: Vec<(Instant, T)>,
}

impl<T> TimedQueue<T> {
    /// Creates an empty [`TimedQueue`] with default capacity.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            stack_1: Vec::new(),
            stack_2: Vec::new(),
        }
    }

    /// Creates an empty [`TimedQueue`] for at least `capacity` elements.
    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self {
            ttl,
            stack_1: Vec::with_capacity(capacity),
            stack_2: Vec::with_capacity(capacity),
        }
    }

    /// Pushes an element to the end of the queue.
    pub fn push_back(&mut self, element: T) {
        self.stack_1.push((Instant::now(), element));
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
        self.ensure_stack_full();
        self.stack_2.pop()
    }

    /// Similar to [`pop_front`](Self::pop_front) but without removing the element.
    pub fn peek_front(&mut self) -> Option<&(Instant, T)> {
        self.ensure_stack_full();
        self.stack_2.first()
    }

    fn ensure_stack_full(&mut self) {
        if self.stack_2.is_empty() {
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
        self.stack_1.len() + self.stack_2.len()
    }

    /// Returns `true` if the queue is definitely empty or `false` if the queue is
    /// possibly empty.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    pub fn is_empty(&self) -> bool {
        self.stack_1.is_empty() && self.stack_2.is_empty()
    }

    /// Refreshes the queue and returns the number of currently contained elements.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_works() {
        let mut queue = TimedQueue::new(Duration::from_millis(50));
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
}
