//! Property tests: random operation sequences checked against a simple model.

use crate::TtlQueue;
use proptest::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;
use timebridge::Timestamp;
use timebridge_clock::ManualClock;

/// One step applied to both the queue and the model.
#[derive(Debug, Clone)]
enum Op {
    /// Advance the clock by this many milliseconds, then push a new element.
    Push(u8),
    /// Pop the front element.
    Pop,
    /// Advance the clock by this many milliseconds, then refresh.
    Refresh(u8),
}

/// Strategy for a single [`Op`].
fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => any::<u8>().prop_map(Op::Push),
        1 => Just(Op::Pop),
        2 => any::<u8>().prop_map(Op::Refresh),
    ]
}

/// Reference model: a list of `(push offset in ms, value)` pairs, oldest first.
#[derive(Debug, Default)]
struct Model {
    /// The elements still in the queue.
    entries: VecDeque<(u64, u32)>,
}

impl Model {
    /// Drops every entry with `now - offset >= ttl` and returns how many remain.
    fn refresh(&mut self, now_ms: u64, ttl_ms: u64) -> usize {
        self.entries.retain(|(at, _)| now_ms - at < ttl_ms);
        self.entries.len()
    }

    /// The expected [`TtlQueue::avg_delta`].
    fn avg_delta(&self) -> Duration {
        match (self.entries.front(), self.entries.back()) {
            (Some(first), Some(last)) if self.entries.len() > 1 => {
                let gaps = u32::try_from(self.entries.len() - 1).expect("small test queues");
                Duration::from_millis(last.0 - first.0) / gaps
            }
            _ => Duration::ZERO,
        }
    }
}

proptest! {
    /// The queue matches the model after every operation.
    #[test]
    fn matches_model(ttl_ms in 0_u64..300, ops in prop::collection::vec(op(), 0..64)) {
        let clock = ManualClock::new(Timestamp::UNIX_EPOCH);
        let mut queue = TtlQueue::with_clock(Duration::from_millis(ttl_ms), &clock);
        let mut model = Model::default();
        let mut clock_ms = 0_u64;

        for (value, op) in (0_u32..).zip(ops) {
            match op {
                Op::Push(gap) => {
                    clock_ms += u64::from(gap);
                    clock.advance(Duration::from_millis(u64::from(gap)));
                    queue.push_back(value);
                    model.entries.push_back((clock_ms, value));
                }
                Op::Pop => {
                    let popped = queue.pop_front().map(|(_, v)| v);
                    prop_assert_eq!(popped, model.entries.pop_front().map(|(_, v)| v));
                }
                Op::Refresh(gap) => {
                    clock_ms += u64::from(gap);
                    clock.advance(Duration::from_millis(u64::from(gap)));
                    prop_assert_eq!(queue.refresh(), model.refresh(clock_ms, ttl_ms));
                }
            }

            let values: Vec<u32> = queue.iter().map(|(_, v)| *v).collect();
            let expected: Vec<u32> = model.entries.iter().map(|(_, v)| *v).collect();
            prop_assert_eq!(values, expected);
            prop_assert_eq!(queue.len(), model.entries.len());
            prop_assert_eq!(queue.is_empty(), model.entries.is_empty());
            prop_assert_eq!(
                queue.peek_front().map(|(_, v)| *v),
                model.entries.front().map(|(_, v)| *v)
            );
            prop_assert_eq!(queue.avg_delta(), model.avg_delta());
        }
    }
}
