use sha2::{Digest, Sha256};

/// Generate a single 32-byte PRNG block.
///
/// `block(seed, ctr) = SHA256(seed || BE32(ctr))`
///
/// The counter is 32-bit to match the Elixir spec: `<<seed::binary, ctr::32-big>>`.
pub fn block(seed: &[u8; 32], ctr: u32) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(ctr.to_be_bytes());
    hasher.finalize().into()
}

/// Generate a uniform random integer in [0, n) using rejection sampling.
///
/// Returns `(value, next_counter)`.
pub fn random_integer(seed: &[u8; 32], ctr: u32, n: u64) -> (u64, u32) {
    // We need 256-bit arithmetic for the rejection sampling limit.
    // 2^256 / n * n is the limit; values >= limit are rejected.
    //
    // Since Rust doesn't have native 256-bit integers, we use u128 pairs
    // or a simplified approach. The Elixir version uses arbitrary precision.
    //
    // We compute: limit = (2^256 / n) * n
    // For the comparison, we interpret the 32-byte hash as a big-endian 256-bit uint.

    let limit = compute_limit_256(n);
    do_random_integer(seed, ctr, n, &limit)
}

fn do_random_integer(seed: &[u8; 32], ctr: u32, n: u64, limit: &[u8; 32]) -> (u64, u32) {
    let hash = block(seed, ctr);

    if ge_256(&hash, limit) {
        do_random_integer(
            seed,
            ctr.checked_add(1).expect("PRNG counter exhausted"),
            n,
            limit,
        )
    } else {
        let value = mod_256(&hash, n);
        (value, ctr.checked_add(1).expect("PRNG counter exhausted"))
    }
}

/// Compute floor(2^256 / n) * n as a 256-bit big-endian byte array.
fn compute_limit_256(n: u64) -> [u8; 32] {
    // 2^256 mod n == (2^256 - n * floor(2^256/n))
    // limit = 2^256 - (2^256 mod n)
    //
    // 2^256 mod n can be computed as:
    // Since 2^256 = (2^256 - 1) + 1, and (2^256 - 1) is all 0xFF bytes:
    // 2^256 mod n = ((2^256 - 1) mod n + 1) mod n
    //
    // (2^256 - 1) mod n can be computed by processing each byte.

    let max_mod_n = {
        // Compute (2^256 - 1) mod n byte by byte (big-endian)
        let mut remainder: u128 = 0;
        for _ in 0..32 {
            remainder = (remainder * 256 + 255) % (n as u128);
        }
        ((remainder + 1) % (n as u128)) as u64
    };

    if max_mod_n == 0 {
        // 2^256 is evenly divisible by n, limit = 2^256 which we represent as [0; 32]
        // But since no 256-bit value can equal 2^256, all values pass.
        // We use all zeros, and the ge_256 check will never trigger (nothing >= 2^256).
        // Actually, we need limit = 2^256, but we can't represent it in 32 bytes.
        // Since all values < 2^256, no rejection ever happens. Return all 0s and
        // adjust ge_256 to treat [0;32] as "no rejection".
        [0u8; 32]
    } else {
        // limit = 2^256 - max_mod_n
        // = [0xFF; 32] + 1 - max_mod_n
        // = [0xFF; 32] - (max_mod_n - 1)
        //
        // Since max_mod_n fits in u64, the subtraction only affects the low 8 bytes.
        // The upper 24 bytes remain 0xFF (no borrow possible since u64::MAX >= sub).
        let sub = max_mod_n - 1;
        let mut result = [0xFFu8; 32];
        let low = u64::MAX - sub;
        result[24..32].copy_from_slice(&low.to_be_bytes());
        result
    }
}

/// Compare two 256-bit big-endian values: a >= b
fn ge_256(a: &[u8; 32], b: &[u8; 32]) -> bool {
    // Special case: if b is all zeros, that means limit = 2^256 (no rejection).
    if b == &[0u8; 32] {
        return false;
    }
    a >= b
}

/// Compute a 256-bit big-endian value mod n (where n fits in u64).
fn mod_256(bytes: &[u8; 32], n: u64) -> u64 {
    let mut remainder: u128 = 0;
    for &byte in bytes {
        remainder = (remainder * 256 + byte as u128) % (n as u128);
    }
    remainder as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_sha256_seed_counter() {
        let seed = [0u8; 32];
        let result = block(&seed, 0);
        // SHA256(32 zero bytes || 4 zero bytes) = SHA256(36 zero bytes)
        let mut hasher = Sha256::new();
        hasher.update([0u8; 36]);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(result, expected);
    }

    #[test]
    fn different_counters_produce_different_blocks() {
        let seed = [0u8; 32];
        assert_ne!(block(&seed, 0), block(&seed, 1));
    }

    #[test]
    fn random_integer_in_range() {
        let seed = [0u8; 32];
        for n in 1..=20 {
            let (val, _) = random_integer(&seed, 0, n);
            assert!(val < n);
        }
    }

    #[test]
    fn random_integer_deterministic() {
        let seed = [0u8; 32];
        let (a, ca) = random_integer(&seed, 0, 100);
        let (b, cb) = random_integer(&seed, 0, 100);
        assert_eq!(a, b);
        assert_eq!(ca, cb);
    }

    #[test]
    fn random_integer_n_equals_1() {
        let seed = [0u8; 32];
        let (val, _) = random_integer(&seed, 0, 1);
        assert_eq!(val, 0);
    }

    #[test]
    fn compute_limit_256_divisible_by_n() {
        // The limit must be divisible by n for all n values.
        // This invariant ensures rejection sampling has no modulo bias.
        for n in [1, 2, 3, 255, 256, 257, 300, 500, 1000, 10000, u64::MAX] {
            let limit = compute_limit_256(n);
            if limit == [0u8; 32] {
                // Special case: limit = 2^256, all values accepted
                continue;
            }
            assert_eq!(
                mod_256(&limit, n),
                0,
                "compute_limit_256({n}) produced a limit not divisible by {n}"
            );
        }
    }

    #[test]
    fn random_integer_large_n() {
        // Regression: pools > 256 entries caused incorrect rejection limits
        let seed = [0u8; 32];
        for n in [300, 500, 1000, 5000] {
            let (val, _) = random_integer(&seed, 0, n);
            assert!(val < n, "random_integer with n={n} returned {val} >= {n}");
        }
    }
}
