# ttl-queue

A queue that drops its content after a given amount of time.

Expired elements are removed when you call `refresh` or `refresh_and_push_back`;
`len`, `iter` and `avg_delta` also see elements that have expired since the last refresh.

## Clocks

Time comes from a [`MonotonicClock`] from [timebridge]. `TtlQueue::new` uses
[`SystemClock`] ([`std::time::Instant`]); `TtlQueue::with_clock(ttl, clock)` takes any
other clock, such as a manual clock in tests. Both are re-exported as
`ttl_queue::MonotonicClock` and `ttl_queue::SystemClock`.

## Crate Features

* `tokio` - Adds `TokioClock`, backed by [`tokio::time::Instant`], so paused tokio time
  drives expiry: `TtlQueue::with_clock(ttl, TokioClock)`.

## Example

To implement an FPS counter, you could use the following technique:

```rust
use std::thread;
use std::time::Duration;
use ttl_queue::TtlQueue;

let mut fps_counter = TtlQueue::new(Duration::from_secs(1));

for _ in 0..50 {
    // Register a new frame and return the number of frames observed
    // within the last second.
    let fps = fps_counter.refresh_and_push_back(());
    assert!(fps >= 1);

    // Sleep ~20 ms to achieve a ~50 Hz frequency.
    thread::sleep(Duration::from_millis(20));
}

let fps = fps_counter.refresh();
let delta = fps_counter.avg_delta();
println!("{fps} frames in the last second, {delta:?} between frames");
```

[`tokio::time::Instant`]: https://docs.rs/tokio/latest/tokio/time/struct.Instant.html
[`std::time::Instant`]: https://doc.rust-lang.org/std/time/struct.Instant.html
[`MonotonicClock`]: https://docs.rs/timebridge/latest/timebridge/trait.MonotonicClock.html
[`SystemClock`]: https://docs.rs/timebridge-clock/latest/timebridge_clock/struct.SystemClock.html
[timebridge]: https://crates.io/crates/timebridge
