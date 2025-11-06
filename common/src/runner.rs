use crate::{
    algos::{et::eigen_trust_run, sr::sybil_rank_run},
    merkle::{self, fixed::DenseMerkleTree, hash_leaf, Hash},
    ScoreEntry, TrustEntry,
};
use getset::Getters;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use sha3::Keccak256;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tracing::info;

/// Local trust object.
///
/// The local trust object stores the trust values that a node assigns to its
/// peers.
///
/// It also stores the sum of the trust values assigned to all peers.
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct OutboundLocalTrust {
    /// The trust values that a node assigns to its peers.
    ///
    /// The `outbound_trust_scores` vector stores the trust values that a node
    /// assigns to its peers. The trust values are represented as a vector of
    /// floats, where each element in the vector corresponds to the trust value
    /// assigned to a particular peer.
    outbound_trust_scores: BTreeMap<u64, f32>,
    /// The sum of the trust values assigned to all peers.
    ///
    /// The `outbound_sum` value stores the sum of the trust values assigned to
    /// all peers. The sum is used to normalize the trust values such that they
    /// add up to 1.
    outbound_sum: f32,
}

impl Default for OutboundLocalTrust {
    fn default() -> Self {
        Self::new()
    }
}

impl OutboundLocalTrust {
    pub fn new() -> Self {
        Self {
            outbound_trust_scores: BTreeMap::new(),
            outbound_sum: 0.0,
        }
    }

    pub fn set_outbound_trust_scores(&mut self, outbound_trust_scores: BTreeMap<u64, f32>) {
        self.outbound_trust_scores = outbound_trust_scores;
        self.outbound_sum = self.outbound_trust_scores.values().sum();
    }

    pub fn from_score_map(score_map: &BTreeMap<u64, f32>) -> Self {
        let outbound_trust_scores = score_map.clone();
        let outbound_sum = outbound_trust_scores.values().sum();
        Self {
            outbound_trust_scores,
            outbound_sum,
        }
    }

    pub fn norm(&self) -> Self {
        let mut outbound_trust_scores = self.outbound_trust_scores.clone();
        for (_, score) in outbound_trust_scores.iter_mut() {
            *score /= self.outbound_sum;
        }
        let outbound_sum = 1.0;
        OutboundLocalTrust {
            outbound_trust_scores,
            outbound_sum,
        }
    }

    /*----------------- BTreeMap similar utils -----------------*/
    pub fn get(&self, peer_id: &u64) -> Option<f32> {
        self.outbound_trust_scores.get(peer_id).copied()
    }

    pub fn contains_key(&self, peer_id: &u64) -> bool {
        self.outbound_trust_scores.contains_key(peer_id)
    }

    pub fn remove(&mut self, peer_id: &u64) {
        let to_be_removed = self
            .outbound_trust_scores
            .get(peer_id)
            .copied()
            .unwrap_or(0.0);
        self.outbound_sum -= to_be_removed;
        self.outbound_trust_scores.remove(peer_id);
    }

    pub fn insert(&mut self, peer_id: u64, value: f32) {
        let prev_value = self
            .outbound_trust_scores
            .get(&peer_id)
            .copied()
            .unwrap_or(0.0);
        self.outbound_sum -= prev_value;
        self.outbound_sum += value;
        self.outbound_trust_scores.insert(peer_id, value);
    }
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct ComputeRunner {
    count: u64,
    indices: HashMap<String, u64>,
    rev_indices: HashMap<u64, String>,
    local_trust: BTreeMap<u64, OutboundLocalTrust>,
    seed_trust: BTreeMap<u64, f32>,
    compute_tree: Option<DenseMerkleTree<Keccak256>>,
    compute_results: Vec<(u64, f32)>,
}

impl ComputeRunner {
    pub fn new() -> Self {
        Self {
            count: 0,
            indices: HashMap::new(),
            rev_indices: HashMap::new(),
            local_trust: BTreeMap::new(),
            seed_trust: BTreeMap::new(),
            compute_tree: None,
            compute_results: Vec::new(),
        }
    }

    pub fn update_trust_map(&mut self, trust_entries: Vec<TrustEntry>) -> Result<(), Error> {
        for entry in trust_entries {
            let from_index = if let Some(i) = self.indices.get(entry.from()) {
                *i
            } else {
                let curr_count = self.count;
                self.indices.insert(entry.from().clone(), curr_count);
                self.rev_indices.insert(curr_count, entry.from().clone());
                self.count += 1;
                curr_count
            };
            let to_index = if let Some(i) = self.indices.get(entry.to()) {
                *i
            } else {
                let curr_count = self.count;
                self.indices.insert(entry.to().clone(), curr_count);
                self.rev_indices.insert(curr_count, entry.to().clone());
                self.count += 1;
                curr_count
            };

            let from_map = self
                .local_trust
                .entry(from_index)
                .or_insert(OutboundLocalTrust::new());
            let is_zero = entry.value() == &0.0;
            let exists = from_map.contains_key(&to_index);
            if is_zero && exists {
                from_map.remove(&to_index);
            } else if !is_zero {
                from_map.insert(to_index, *entry.value());
            }
        }
        info!("LT_MAP_UPDATE");

        Ok(())
    }

    pub fn update_seed_map(&mut self, seed_entries: Vec<ScoreEntry>) -> Result<(), Error> {
        for entry in seed_entries {
            let index = if let Some(i) = self.indices.get(entry.id()) {
                *i
            } else {
                let curr_count = self.count;
                self.indices.insert(entry.id().clone(), curr_count);
                self.rev_indices.insert(curr_count, entry.id().clone());
                self.count += 1;
                curr_count
            };
            let is_zero = entry.value() == &0.0;
            let exists = self.seed_trust.contains_key(&index);
            if is_zero && exists {
                self.seed_trust.remove(&index);
            } else if !is_zero {
                self.seed_trust.insert(index, *entry.value());
            }
        }
        info!("ST_MAP_UPDATE");

        Ok(())
    }

    /// Compute the EigenTrust scores.
    pub fn compute_et(&mut self, alpha: Option<f32>, delta: Option<f32>) -> Result<(), Error> {
        info!("COMPUTE_RUN_ET");
        let res = eigen_trust_run(
            self.local_trust.clone(),
            self.seed_trust.clone(),
            self.count,
            alpha,
            delta,
        );
        self.compute_results = res;
        Ok(())
    }

    /// Compute the SybilRank scores.
    pub fn compute_sr(&mut self, walk_length: Option<u32>) -> Result<(), Error> {
        info!("COMPUTE_RUN_SR");
        let res = sybil_rank_run(
            self.local_trust.clone(),
            self.seed_trust.clone(),
            self.count,
            walk_length,
        );
        self.compute_results = res;
        Ok(())
    }

    /// Create the compute tree.
    pub fn create_compute_tree(&mut self) -> Result<(), Error> {
        info!("CREATE_COMPUTE_TREE");
        let score_hashes: Vec<Hash> = self
            .compute_results
            .par_iter()
            .map(|(_, x)| hash_leaf::<Keccak256>(x.to_be_bytes().to_vec()))
            .collect();
        let compute_tree =
            DenseMerkleTree::<Keccak256>::new(score_hashes).map_err(Error::Merkle)?;
        info!(
            "COMPUTE_TREE_ROOT_HASH: {}",
            compute_tree.root().map_err(Error::Merkle)?
        );
        self.compute_tree = Some(compute_tree);
        Ok(())
    }

    /// Get the compute scores.
    pub fn get_compute_scores(&self) -> Result<Vec<ScoreEntry>, Error> {
        let index_to_address: HashMap<&u64, &String> =
            self.indices.iter().map(|(k, v)| (v, k)).collect();

        let mut entries = Vec::new();
        for (index, val) in &self.compute_results {
            let address = index_to_address
                .get(index)
                .ok_or(Error::IndexToAddressNotFound(*index))?;
            let score_entry = ScoreEntry::new((*address).clone(), *val);
            entries.push(score_entry);
        }
        Ok(entries)
    }

    /// Get the compute tree root hash.
    pub fn get_root_hash(&self) -> Result<Hash, Error> {
        let ct_tree_root = self
            .compute_tree
            .as_ref()
            .map(|ct| ct.root())
            .unwrap()
            .map_err(Error::Merkle)?;
        Ok(ct_tree_root)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("'local_trust_sub_trees' not found for index: {0}")]
    LocalTrustSubTreesNotFoundWithIndex(u64),
    #[error("'domain_index' not found for address: {0}")]
    DomainIndexNotFound(String),
    #[error("Merkle Error: {0}")]
    Merkle(merkle::Error),
    #[error("Misc Error: {0}")]
    Misc(String),
    /// The index to address mapping for the domain are not found.
    #[error("IndexToAddressNotFound Error: {0}")]
    IndexToAddressNotFound(u64),
}
