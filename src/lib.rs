mod prng;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Winner {
    pub position: u32,
    pub entry_id: String,
}

/// Run a deterministic draw.
///
/// Given a list of entries, a 32-byte seed, and a winner count,
/// produces an ordered list of winners. Same inputs always produce the same output.
pub fn draw(entries: &[Entry], seed: &[u8; 32], count: u32) -> Result<Vec<Winner>, String> {
    validate_entries(entries)?;

    let pool = expand_pool(entries);
    let shuffled = shuffle(&pool, seed);

    let mut winners = Vec::new();
    let mut seen = HashSet::new();

    for id in &shuffled {
        if seen.insert(id.as_str()) {
            winners.push(id.clone());
            if winners.len() == count as usize {
                break;
            }
        }
    }

    Ok(winners
        .into_iter()
        .enumerate()
        .map(|(i, id)| Winner {
            position: (i + 1) as u32,
            entry_id: id,
        })
        .collect())
}

/// Sort entries by id (ascending lexicographic) and expand into a flat pool.
/// Each entry with weight N produces N consecutive copies of its id.
fn expand_pool(entries: &[Entry]) -> Vec<String> {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    sorted
        .iter()
        .flat_map(|e| std::iter::repeat_n(e.id.clone(), e.weight as usize))
        .collect()
}

/// Durstenfeld (modern Fisher-Yates) shuffle using counter-mode SHA256 PRNG.
fn shuffle(pool: &[String], seed: &[u8; 32]) -> Vec<String> {
    let mut arr: Vec<String> = pool.to_vec();
    let m = arr.len();

    if m <= 1 {
        return arr;
    }

    let mut ctr: u32 = 0;

    for i in (1..m).rev() {
        let (j, next_ctr) = prng::random_integer(seed, ctr, (i + 1) as u64);
        arr.swap(i, j as usize);
        ctr = next_ctr;
    }

    arr
}

fn validate_entries(entries: &[Entry]) -> Result<(), String> {
    if entries.is_empty() {
        return Err("entries must not be empty".to_string());
    }

    let mut ids = HashSet::new();
    for entry in entries {
        if entry.weight == 0 {
            return Err(format!("entry weight must be positive: {}", entry.id));
        }
        if !ids.insert(&entry.id) {
            return Err("entries must not contain duplicate ids".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
