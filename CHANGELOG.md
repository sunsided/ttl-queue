# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Breaking: removed the `vecdeque` and `doublestack` features; `VecDeque` is always used.
  The `doublestack` backend returned elements in the wrong order.
- Breaking: `peek_front` now takes `&self`, and `iter` returns `std::collections::vec_deque::Iter`.
- Upgraded to Rust edition 2024 (MSRV 1.85).
- `avg_delta` now runs in O(1).

### Fixed

- Building with `default-features = false` (e.g. with only the `tokio` feature) works again.

### Added

- `IntoIterator` for `&TtlQueue<T>`.

## [0.2.0] - 2023-08-02

### Added

- Added `iter()` and `IntoIter` support.
- Added `avg_delta()` to calculate duration differences directly. 

## 0.1.0 - 2023-08-02

### Added

- 🎉 Initial release.

[0.2.0]: https://github.com/sunsided/ttl-queue/releases/tag/0.2.0
