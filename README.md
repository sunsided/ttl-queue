# Timed Queue

A queue that drops its content after a given amount of time.

## Example

To implement an FPS counter, you could use the following technique:

```
use std::thread;
use std::time::Duration;
use timed_queue::TimedQueue;
let mut fps_counter = TimedQueue::new(Duration::from_secs_f64(1.0));

for i in 0..100 {
    // Register a new frame and return the number of frames observed
    // within the last second.
    let fps = fps_counter.refresh_and_push_back(());
    debug_assert!(fps >= 1);

    // Sleep 10 ms to achieve a ~100 Hz frequency.
    thread::sleep(Duration::from_millis(10));
}

let fps = fps_counter.refresh();
debug_assert!(fps >= 95 && fps <= 105);
```
