use super::*;

fn seed_from_hex(hex_str: &str) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for (i, chunk) in hex_str.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk).unwrap();
        bytes[i] = u8::from_str_radix(s, 16).unwrap();
    }
    bytes
}

// --- Algorithm vector A-1: minimal draw ---

#[test]
fn vector_a1_minimal_draw() {
    let entries = vec![
        Entry {
            id: "a".into(),
            weight: 1,
        },
        Entry {
            id: "b".into(),
            weight: 1,
        },
        Entry {
            id: "c".into(),
            weight: 1,
        },
    ];
    let seed = [0u8; 32];
    let result = draw(&entries, &seed, 1).unwrap();

    assert_eq!(
        result,
        vec![Winner {
            position: 1,
            entry_id: "c".into()
        }]
    );
}

// --- Algorithm vector A-2: weighted entries ---

#[test]
fn vector_a2_weighted_entries() {
    let entries = vec![
        Entry {
            id: "alpha".into(),
            weight: 3,
        },
        Entry {
            id: "beta".into(),
            weight: 1,
        },
        Entry {
            id: "gamma".into(),
            weight: 2,
        },
    ];
    let seed = [0xFF; 32];
    let result = draw(&entries, &seed, 2).unwrap();

    assert_eq!(
        result,
        vec![
            Winner {
                position: 1,
                entry_id: "gamma".into()
            },
            Winner {
                position: 2,
                entry_id: "alpha".into()
            },
        ]
    );
}

// --- Algorithm vector A-3: deduplication ---

#[test]
fn vector_a3_deduplication() {
    let entries = vec![
        Entry {
            id: "x".into(),
            weight: 5,
        },
        Entry {
            id: "y".into(),
            weight: 1,
        },
    ];
    let seed = seed_from_hex("ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789");
    let result = draw(&entries, &seed, 2).unwrap();

    assert_eq!(
        result,
        vec![
            Winner {
                position: 1,
                entry_id: "x".into()
            },
            Winner {
                position: 2,
                entry_id: "y".into()
            },
        ]
    );
}

// --- Algorithm vector A-4: count exceeds unique entries ---

#[test]
fn vector_a4_count_exceeds_unique() {
    let entries = vec![Entry {
        id: "solo".into(),
        weight: 3,
    }];
    let seed = [0x11; 32];
    let result = draw(&entries, &seed, 5).unwrap();

    assert_eq!(
        result,
        vec![Winner {
            position: 1,
            entry_id: "solo".into()
        }]
    );
}

// --- Algorithm vector A-5: single entry ---

#[test]
fn vector_a5_single_entry() {
    let entries = vec![Entry {
        id: "only".into(),
        weight: 1,
    }];
    let seed = [0x22; 32];
    let result = draw(&entries, &seed, 1).unwrap();

    assert_eq!(
        result,
        vec![Winner {
            position: 1,
            entry_id: "only".into()
        }]
    );
}

// --- Unit tests ---

#[test]
fn deterministic_same_inputs_same_output() {
    let entries = vec![
        Entry {
            id: "a".into(),
            weight: 1,
        },
        Entry {
            id: "b".into(),
            weight: 1,
        },
        Entry {
            id: "c".into(),
            weight: 1,
        },
    ];
    let seed = [0u8; 32];
    let r1 = draw(&entries, &seed, 2).unwrap();
    let r2 = draw(&entries, &seed, 2).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn different_seeds_different_output() {
    let entries = vec![
        Entry {
            id: "a".into(),
            weight: 1,
        },
        Entry {
            id: "b".into(),
            weight: 1,
        },
        Entry {
            id: "c".into(),
            weight: 1,
        },
    ];
    let seed1 = [0u8; 32];
    let seed2 = [1u8; 32];
    let r1 = draw(&entries, &seed1, 3).unwrap();
    let r2 = draw(&entries, &seed2, 3).unwrap();
    // Same entry_ids but different order (very likely)
    let ids1: Vec<&str> = r1.iter().map(|w| w.entry_id.as_str()).collect();
    let ids2: Vec<&str> = r2.iter().map(|w| w.entry_id.as_str()).collect();
    assert_ne!(ids1, ids2);
}

#[test]
fn empty_entries_returns_error() {
    let result = draw(&[], &[0u8; 32], 1);
    assert!(result.is_err());
}

#[test]
fn duplicate_ids_returns_error() {
    let entries = vec![
        Entry {
            id: "a".into(),
            weight: 1,
        },
        Entry {
            id: "a".into(),
            weight: 2,
        },
    ];
    let result = draw(&entries, &[0u8; 32], 1);
    assert!(result.is_err());
}
