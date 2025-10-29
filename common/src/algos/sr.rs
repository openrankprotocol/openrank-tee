use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{BTreeMap, HashSet},
    time::Instant,
};
use tracing::info;

use crate::runner::OutboundLocalTrust;

/// The number of random walk steps to perform in the Sybil Rank algorithm.
const WALK_LENGTH: u32 = 10;

/// The number of random walks to perform from each seed node.
const NUM_WALKS: u32 = 1000;

fn find_reachable_peers(
    lt: &BTreeMap<u64, OutboundLocalTrust>,
    seed: &BTreeMap<u64, f32>,
) -> HashSet<u64> {
    let mut to_visit: Vec<&u64> = seed.keys().collect();
    let mut visited = HashSet::new();
    while let Some(i) = to_visit.pop() {
        if visited.contains(i) {
            continue;
        }
        visited.insert(*i);
        for (j, v) in lt.get(i).unwrap().outbound_trust_scores() {
            if !visited.contains(j) && *v > 0.0 {
                to_visit.push(j);
            }
        }
    }
    visited
}

/// Pre-processes a mutable local trust matrix `lt` by modifying it in-place:
///
/// - Removes self-trust (diagonal entries), as prohibited by Sybil Rank.
/// - Ensures all nodes have outbound trust, redistributing to seed peers if necessary.
fn pre_process(
    lt: &mut BTreeMap<u64, OutboundLocalTrust>,
    seed: &mut BTreeMap<u64, f32>,
    count: u64,
) {
    // Calculate the sum of all seed trust values.
    let sum: f32 = seed.par_iter().map(|(_, v)| v).sum();

    if sum == 0.0 {
        for i in 0..count {
            seed.insert(i, 1.0);
        }
    }

    for from in 0..count {
        let sum = lt.get(&from).map(|lt| lt.outbound_sum()).unwrap_or(&0.0);
        // If peer does not have outbound trust,
        // his trust will be distributed to seed peers based on their seed/pre-trust
        if *sum == 0.0 {
            let single_lt = OutboundLocalTrust::from_score_map(seed);
            lt.insert(from, single_lt);
        }
    }

    let reachable = find_reachable_peers(lt, seed);
    lt.retain(|from, _| reachable.contains(from));
}

/// Normalizes the `lt` matrix by dividing each element by the sum of its row.
fn normalise_lt(lt: &BTreeMap<u64, OutboundLocalTrust>) -> BTreeMap<u64, OutboundLocalTrust> {
    lt.par_iter()
        .fold(BTreeMap::new, |mut lt_norm, (from, from_map)| {
            let from_map_norm = from_map.norm();
            lt_norm.insert(*from, from_map_norm);
            lt_norm
        })
        .reduce(BTreeMap::new, |mut acc, lt_norm| {
            acc.extend(lt_norm);
            acc
        })
}

/// Normalizes the scores, to eliminate the rounding error
fn normalise_scores(scores: &BTreeMap<u64, f32>) -> BTreeMap<u64, f32> {
    // Calculate the sum of all seed trust values.
    let sum: f32 = scores.par_iter().map(|(_, v)| v).sum();

    if sum == 0.0 {
        return scores.clone();
    }

    scores
        .par_iter()
        .fold(BTreeMap::new, |mut scores, (i, value)| {
            scores.insert(*i, *value / sum);
            scores
        })
        .reduce(BTreeMap::new, |mut acc, scores| {
            acc.extend(scores);
            acc
        })
}

/// Performs a single deterministic walk step following the trust edges.
/// This is the core of SybilRank - no damping/restart, just pure edge following.
/// Returns the new distribution after one step of the random walk.
fn fixed_walk_step(
    lt: &BTreeMap<u64, OutboundLocalTrust>,
    current_scores: &BTreeMap<u64, f32>,
) -> BTreeMap<u64, f32> {
    // Compute the distribution after following edges (no restart probability)
    let next_scores = lt
        .par_iter()
        .map(|(from, from_map)| {
            let origin_score = current_scores.get(from).unwrap_or(&0.0);
            let mut partial = BTreeMap::new();
            for (to, value) in from_map.outbound_trust_scores() {
                let score = *value * origin_score;
                let to_score = partial.get(to).unwrap_or(&0.0);
                partial.insert(*to, to_score + score);
            }
            partial
        })
        .reduce(
            || BTreeMap::new(),
            |mut acc, partial| {
                for (k, v) in partial {
                    *acc.entry(k).or_insert(0.0) += v;
                }
                acc
            },
        );

    next_scores
}

/// Performs the Sybil Rank algorithm on the given local trust matrix (`lt`) and seed trust values (`seed`).
/// The algorithm performs random walks of exactly `walk_length` steps from seed nodes.
/// The key insight is that walks from honest nodes stay in honest regions, while walks from
/// Sybil nodes spread more broadly, allowing discrimination between honest and Sybil nodes.
/// It returns a vector of tuples containing the node ID and the final score.
pub fn sybil_rank_run(
    mut lt: BTreeMap<u64, OutboundLocalTrust>,
    mut seed: BTreeMap<u64, f32>,
    count: u64,
    walk_length: Option<u32>,
    num_walks: Option<u32>,
) -> Vec<(u64, f32)> {
    let start = Instant::now();
    let walk_len = walk_length.unwrap_or(WALK_LENGTH);
    let walks = num_walks.unwrap_or(NUM_WALKS);

    info!("WALK_LENGTH: {}", walk_len);
    info!("NUM_WALKS: {}", walks);
    info!(
        "PRE_PROCESS_START, LT_SIZE: {}, SEED_SIZE: {}",
        lt.len(),
        seed.len()
    );

    pre_process(&mut lt, &mut seed, count);
    info!(
        "PRE_PROCESS_FINISH: {:?}, LT_SIZE: {}, SEED_SIZE: {}",
        start.elapsed(),
        lt.len(),
        seed.len()
    );

    info!("NORMALISE_LT_SEED");
    seed = normalise_scores(&seed);
    lt = normalise_lt(&lt);

    info!("SYBIL_RANK_START");
    let start = Instant::now();

    // Accumulate landing probabilities across all walks
    let mut accumulated_scores: BTreeMap<u64, f32> = BTreeMap::new();

    for walk_id in 0..walks {
        let mut current_scores = seed.clone();

        // Perform exactly walk_len steps - no convergence checking
        for _step in 0..walk_len {
            current_scores = fixed_walk_step(&lt, &current_scores);
            current_scores = normalise_scores(&current_scores);
        }

        // Accumulate the final scores after exactly walk_len steps
        for (node, score) in current_scores {
            *accumulated_scores.entry(node).or_insert(0.0) += score;
        }

        if (walk_id + 1) % 100 == 0 {
            info!("COMPLETED_WALKS: {}/{}", walk_id + 1, walks);
        }
    }

    // Average the accumulated scores
    let final_scores: BTreeMap<u64, f32> = accumulated_scores
        .into_iter()
        .map(|(node, total_score)| (node, total_score / walks as f32))
        .collect();

    let final_scores = normalise_scores(&final_scores);

    info!(
        "SYBIL_RANK_END: {:?}, NUM_SCORES: {}, WALK_LENGTH: {}, NUM_WALKS: {}",
        start.elapsed(),
        final_scores.len(),
        walk_len,
        walks
    );

    final_scores.into_iter().collect()
}
