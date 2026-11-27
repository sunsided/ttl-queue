# Timed Queue

A queue that drops its content after a given amount of time.

## Crate Features

* `tokio` - Uses [`tokio::time::Instant`] instead of [`std::time::Instant`].

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
