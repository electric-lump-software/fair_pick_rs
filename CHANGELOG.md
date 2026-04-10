# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-04-10

### Fixed

- PRNG counter uses `checked_add` instead of bare `+` for consistent overflow behaviour across build profiles
- Reject `count == 0` in `draw()` instead of returning an empty result

## [0.1.2] - 2026-04-10

### Fixed

- Reject entries with `weight == 0` instead of silently dropping them from the draw pool
- Narrow PRNG counter from `u64` to `u32` to match the Elixir spec (`ctr::32-big`), eliminating silent truncation

## [0.1.0] - 2026-04-04

### Added

- `draw()` function implementing deterministic Fisher-Yates shuffle
- Counter-mode SHA-256 PRNG with rejection sampling
- `Entry` and `Winner` types with serde support
- Test vectors A-1 through A-5 matching the reference implementation
