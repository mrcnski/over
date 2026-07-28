# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.6] - 2026-07-28

No public API changes. Parser behavior is unchanged and was verified
byte-for-byte (values and error messages) against 0.6.5 across the full test
corpus and an adversarial input set.

### Changed

- Parsing is 2-5x faster depending on workload (int-heavy 5.1x, string-heavy
  3.3x, nested 1.8x):
  - Hot loops (whitespace/comments, field names, digits, string bodies) now
    scan bytes in bulk with an ASCII fast path instead of iterating chars
    through `Rc<RefCell<...>>` one at a time.
  - Duplicate field detection uses a `HashSet` instead of an O(n²) scan.
  - Integers with 18 or fewer digits parse through `i64` before conversion
    to `BigInt`.
  - Decimal fractions are constructed with a single rational reduction
    instead of three.
  - Values not followed by an operator skip operator-queue allocations.

### Fixed

- Removed an unsafe self-referential `mem::transmute` in the internal
  character stream (latent unsoundness; no known miscompilation).

### Meta

- Repository moved to <https://github.com/mrcnski/over> (restored from
  Software Heritage with full history).

## [0.6.5] and earlier

Released before this changelog was introduced; see the git history.

[0.6.6]: https://github.com/mrcnski/over/compare/4a1b76d...v0.6.6
