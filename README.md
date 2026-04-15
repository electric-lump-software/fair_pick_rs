# fair_pick_rs

Deterministic, verifiable draw algorithm for provably fair random selection — in Rust.

Part of [Wallop!](https://wallop.run) — a provably fair draw platform where nobody controls the outcome. Not the organiser, not the platform, not us.

This is the Rust implementation of the same algorithm published as [fair_pick](https://github.com/electric-lump-software/fair_pick) in Elixir. Both implementations are tested against shared frozen test vectors — they must produce identical output for the same inputs.

The Rust version compiles to WebAssembly, which powers the [in-browser verification](https://wallop.run/how-verification-works) on every Wallop proof page. When you click "Verify independently" on a proof page, this is the code that runs — right in your browser, no server involved.

## Why?

Every Wallop draw uses public, unpredictable entropy: a [drand](https://drand.love) beacon value and a live weather observation from Middle Wallop, Hampshire — three villages in the Test Valley, and the reason the service is called Wallop. The entropy is combined with the entry list to produce a seed, and that seed is fed into this algorithm.

The algorithm is deterministic. Same inputs, same winners, every time. That's the whole point — anyone can replay a draw and verify the result independently. You don't have to trust us. You don't have to trust anyone.

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

    // 32-byte seed — typically derived from public entropy
    let seed = [0u8; 32];

    let winners = draw(&entries, &seed, 2).unwrap();

    for w in &winners {
        println!("Position {}: {}", w.position, w.entry_id);
    }
}
```

`draw()` returns `Vec<Winner>`, where each `Winner` has a `position` (1-indexed) and `entry_id`. It returns `Err` if entries are empty or contain duplicate IDs.

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

## Determinism and verifiability

The same inputs always produce the same outputs. There is no external state, no clock, and no random number source beyond the seed. Any party can independently reproduce a draw result by running `draw()` with the same entries, seed, and count.

The five canonical test vectors (A-1 through A-5) are included in the test suite and shared with the Elixir implementation. Both must agree.

## Dependencies

- [`sha2`](https://crates.io/crates/sha2) — SHA-256 hash function
- [`serde`](https://crates.io/crates/serde) — serialization support for `Entry` and `Winner`

## License

MIT — see [LICENSE](LICENSE).
