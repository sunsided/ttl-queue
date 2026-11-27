# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-11-28

### Changed

- README: fixed the crate name, documented that expired elements are only removed on
  `refresh`, and described the clock setup.
- Crate keywords now include `expiring` and `sliding-window`.
- Benchmarks use `std::hint::black_box` (criterion 0.8).

## [0.3.0] - 2026-11-28

### Changed

- Breaking: removed the `vecdeque` and `doublestack` features; `VecDeque` is always used.
  The `doublestack` backend returned elements in the wrong order.
- Breaking: `peek_front` now takes `&self`, and `iter` returns `std::collections::vec_deque::Iter`.
- Breaking: time comes from a timebridge `MonotonicClock`. `TtlQueue<T, C = SystemClock>`
  keeps `new`/`with_capacity` on the system clock; `with_clock(ttl, clock)` takes any
  other clock.
- Breaking: the `tokio` feature no longer swaps the global `Instant`; it adds `TokioClock`,
  used as `TtlQueue::with_clock(ttl, TokioClock)`.
- Upgraded to Rust edition 2024 (MSRV 1.88, required by timebridge).
- `avg_delta` now runs in O(1).

### Fixed

- Building with `default-features = false` (e.g. with only the `tokio` feature) works again.

### Added

- `IntoIterator` for `&TtlQueue<T>`.
- Property tests (proptest) against a reference model, Kani proofs for `refresh` and
  `avg_delta`, and mutation testing (cargo-mutants), all run in CI.
- CI lints the workflows with actionlint and zizmor. Workflows use least-privilege
  permissions and SHA-pinned actions.
- Dependabot keeps Cargo dependencies and GitHub Actions up to date (weekly, 7-day cooldown).

## [0.2.0] - 2023-08-02

### Added

- Added `iter()` and `IntoIter` support.
- Added `avg_delta()` to calculate duration differences directly. 

## 0.1.0 - 2023-08-02

### Added

- 🎉 Initial release.

[0.3.1]: https://github.com/sunsided/ttl-queue/releases/tag/0.3.1
[0.3.0]: https://github.com/sunsided/ttl-queue/releases/tag/0.3.0
[0.2.0]: https://github.com/sunsided/ttl-queue/releases/tag/0.2.0
