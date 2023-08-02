use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A queue that drops its content after a given amount of time.
#[derive(Debug)]
pub struct TimedQueue<T> {
    ttl: Duration,
    queue: VecDeque<(Instant, T)>,
}

impl<T> TimedQueue<T> {
    /// Creates an empty [`TimedQueue`] with default capacity.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            queue: VecDeque::new(),
        }
    }

    /// Creates an empty [`TimedQueue`] for at least `capacity` elements.
    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self {
            ttl,
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Pushes an element to the end of the queue.
    pub fn push_back(&mut self, element: T) {
        self.queue.push_back((Instant::now(), element));
    }

    /// Pushes an element to the end of the queue and returns the number of items
    /// currently in the queue. This operation is O(N) at worst.
    pub fn refresh_and_push_back(&mut self, element: T) -> usize {
        let count = self.refresh();
        self.queue.push_back((Instant::now(), element));
        count + 1
    }

    /// Gets the element from the front of the queue if it exists, as well as the
    /// time instant at which it was added.
    pub fn pop_front(&mut self) -> Option<(Instant, T)> {
        self.queue.pop_front()
    }

    /// Gets the element from the back of the queue if it exists, as well as the
    /// time instant at which it was added.
    pub fn pop_back(&mut self) -> Option<(Instant, T)> {
        self.queue.pop_back()
    }

    /// Gets the number elements currently in the queue, including potentially expired elements.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if the queue is definitely empty or `false` if the queue is
    /// possibly empty.
    ///
    /// This operation is O(1). In order to obtain an accurate count in O(N) (worst-case),
    /// use [`refresh`](Self::refresh) instead.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Refreshes the queue and returns the number of currently contained elements.
    pub fn refresh(&mut self) -> usize {
        let now = Instant::now();
        while let Some((instant, _element)) = self.queue.front() {
            if (now - *instant) < self.ttl {
                break;
            }

            self.queue.pop_front();
        }
        self.queue.len()
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

        let value = queue.pop_back().unwrap();
        assert_eq!(value.1, 30);

        assert_eq!(queue.refresh(), 1);

        thread::sleep(Duration::from_millis(50));
        assert_eq!(queue.refresh(), 0);
    }
}
