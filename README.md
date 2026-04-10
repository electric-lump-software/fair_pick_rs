# fair_pick_rs

A Rust implementation of a deterministic, verifiable draw algorithm for provably fair random selection. Given a list of weighted entries, a 32-byte seed, and a winner count, `draw()` always produces the same ordered result — making draws independently verifiable by any party with the same inputs.

## Algorithm

1. **Sort** entries by `id` (ascending lexicographic order) for deterministic input ordering.
2. **Expand** each entry into a flat pool: an entry with `weight = N` produces `N` consecutive copies of its `id`.
3. **Shuffle** the pool using a Durstenfeld (modern Fisher-Yates) shuffle driven by a counter-mode SHA-256 PRNG.
4. **Deduplicate** by walking the shuffled pool and keeping the first occurrence of each `id`.
5. **Truncate** to the requested winner count.

### PRNG

The PRNG generates each shuffle index as:

```
block(seed, ctr) = SHA256(seed || BE32(ctr))
```

where `seed` is the 32-byte draw seed and `ctr` is a monotonically incrementing 32-bit counter encoded as big-endian bytes. Each call to `random_integer(seed, ctr, n)` uses rejection sampling to ensure a perfectly uniform distribution over `[0, n)`: if the 256-bit hash value falls in the rejection region (`value >= floor(2^256 / n) * n`), the counter increments and a new block is drawn. In practice, rejection is rare (< 1 in 2^192 for any realistic `n`).

## Getting started

```bash
git clone --recursive https://github.com/electric-lump-software/fair_pick_rs.git
cd fair_pick_rs
cargo test
```

If you've already cloned without `--recursive`:

```bash
git submodule update --init
```

## Usage

```rust
use fair_pick_rs::{draw, Entry};

fn main() {
    let entries = vec![
        Entry { id: "alice".into(), weight: 2 },
        Entry { id: "bob".into(),   weight: 1 },
        Entry { id: "carol".into(), weight: 3 },
    ];

    // 32-byte seed — typically a public commitment value
    let seed = [0u8; 32];

    let winners = draw(&entries, &seed, 2).unwrap();

    for w in &winners {
        println!("Position {}: {}", w.position, w.entry_id);
    }
}
```

`draw()` returns `Vec<Winner>`, where each `Winner` has a `position` (1-indexed) and `entry_id`. It returns `Err` if entries are empty or contain duplicate ids.

## Determinism and Verifiability

The same inputs always produce the same outputs. There is no external state, no clock, and no random number source beyond the seed. Any party can independently reproduce a draw result by running `draw()` with the same entries, seed, and count.

This implementation follows a published specification. The five canonical test vectors (A-1 through A-5) are included in the test suite and cover: minimal draws, weighted entries, deduplication, count exceeding unique entries, and single-entry draws.

## Dependencies

- [`sha2`](https://crates.io/crates/sha2) — SHA-256 hash function
- [`serde`](https://crates.io/crates/serde) — serialization support for `Entry` and `Winner`

## License

MIT — see [LICENSE](LICENSE).
