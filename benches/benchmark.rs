use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;
use timed_queue::TimedQueue;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("push_back (Duration::MAX)", |b| {
        let mut queue = TimedQueue::new(Duration::MAX);
        b.iter(|| {
            queue.push_back(black_box(10));
        })
    });

    c.bench_function("refresh_and_push_back (Duration::MAX)", |b| {
        let mut queue = TimedQueue::new(Duration::MAX);
        b.iter(|| {
            queue.refresh_and_push_back(black_box(10));
        })
    });

    c.bench_function("refresh_and_push_back (Duration::ZERO)", |b| {
        let mut queue = TimedQueue::new(Duration::ZERO);
        b.iter(|| {
            queue.push_back(black_box(10));
        })
    });

    let mut group = c.benchmark_group("push_back, then refresh (Duration::ZERO)");
    for (i, elements) in [100, 1000].iter().enumerate() {
        group.throughput(Throughput::Elements(*elements));
        group.bench_with_input(format!("test {}", i), elements, |b, &elems| {
            let mut queue = TimedQueue::new(Duration::ZERO);
            b.iter(|| {
                for i in 0..elems {
                    queue.push_back(black_box(i));
                }

                queue.refresh();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
