use crate::{
    merkle::{self, hash_leaf, hash_two, incremental::DenseIncrementalMerkleTree, Hash},
    tx::trust::{OwnedNamespace, ScoreEntry, TrustEntry},
    Domain, DomainHash,
};
use getset::Getters;
use serde::{Deserialize, Serialize};
use sha3::Keccak256;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tracing::info;

pub mod compute_runner;
pub mod verification_runner;

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
pub struct BaseRunner {
    count: HashMap<DomainHash, u64>,
    indices: HashMap<DomainHash, HashMap<String, u64>>,
    rev_indices: HashMap<DomainHash, HashMap<u64, String>>,
    local_trust: HashMap<OwnedNamespace, BTreeMap<u64, OutboundLocalTrust>>,
    seed_trust: HashMap<OwnedNamespace, BTreeMap<u64, f32>>,
    lt_sub_trees: HashMap<DomainHash, HashMap<u64, DenseIncrementalMerkleTree<Keccak256>>>,
    lt_master_tree: HashMap<DomainHash, DenseIncrementalMerkleTree<Keccak256>>,
    st_master_tree: HashMap<DomainHash, DenseIncrementalMerkleTree<Keccak256>>,
}

impl BaseRunner {
    pub fn new(domains: &[Domain]) -> Self {
        let mut count = HashMap::new();
        let mut indices = HashMap::new();
        let mut rev_indices = HashMap::new();
        let mut local_trust = HashMap::new();
        let mut seed_trust = HashMap::new();
        let mut lt_sub_trees = HashMap::new();
        let mut lt_master_tree = HashMap::new();
        let mut st_master_tree = HashMap::new();
        let mut compute_results = HashMap::new();
        for domain in domains {
            let domain_hash = domain.to_hash();
            count.insert(domain_hash, 0);
            indices.insert(domain_hash, HashMap::new());
            rev_indices.insert(domain_hash, HashMap::new());
            local_trust.insert(domain.trust_namespace(), BTreeMap::new());
            seed_trust.insert(domain.trust_namespace(), BTreeMap::new());
            lt_sub_trees.insert(domain_hash, HashMap::new());
            lt_master_tree.insert(
                domain_hash,
                DenseIncrementalMerkleTree::<Keccak256>::new(32),
            );
            st_master_tree.insert(
                domain_hash,
                DenseIncrementalMerkleTree::<Keccak256>::new(32),
            );
            compute_results.insert(domain_hash, Vec::<f32>::new());
        }
        Self {
            count,
            indices,
            rev_indices,
            local_trust,
            seed_trust,
            lt_sub_trees,
            lt_master_tree,
            st_master_tree,
        }
    }

    pub fn update_trust(
        &mut self,
        domain: Domain,
        trust_entries: Vec<TrustEntry>,
    ) -> Result<(), Error> {
        let domain_indices = self
            .indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::IndicesNotFound(domain.to_hash()))?;
        let rev_domain_indices = self
            .rev_indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::ReverseIndicesNotFound(domain.to_hash()))?;
        let count = self
            .count
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::CountNotFound(domain.to_hash()))?;
        let lt_sub_trees = self
            .lt_sub_trees
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::LocalTrustSubTreesNotFoundWithDomain(
                domain.to_hash(),
            ))?;
        let lt_master_tree = self
            .lt_master_tree
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::LocalTrustMasterTreeNotFound(domain.to_hash()))?;
        let lt = self
            .local_trust
            .get_mut(&domain.trust_namespace())
            .ok_or::<Error>(Error::LocalTrustNotFound(domain.trust_namespace()))?;
        let default_sub_tree = DenseIncrementalMerkleTree::<Keccak256>::new(32);
        for entry in trust_entries {
            let from_index = if let Some(i) = domain_indices.get(entry.from()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.from().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.from().clone());
                *count += 1;
                curr_count
            };
            let to_index = if let Some(i) = domain_indices.get(entry.to()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.to().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.to().clone());
                *count += 1;
                curr_count
            };

            let from_map = lt.entry(from_index).or_insert(OutboundLocalTrust::new());
            let is_zero = entry.value() == &0.0;
            let exists = from_map.contains_key(&to_index);
            if is_zero && exists {
                from_map.remove(&to_index);
            } else if !is_zero {
                from_map.insert(to_index, *entry.value());
            }

            lt_sub_trees
                .entry(from_index)
                .or_insert_with(|| default_sub_tree.clone());
            let sub_tree = lt_sub_trees
                .get_mut(&from_index)
                .ok_or(Error::LocalTrustSubTreesNotFoundWithIndex(from_index))?;

            let leaf = hash_leaf::<Keccak256>(entry.value().to_be_bytes().to_vec());
            sub_tree.insert_leaf(to_index, leaf);

            let sub_tree_root = sub_tree.root().map_err(Error::Merkle)?;

            let leaf = hash_leaf::<Keccak256>(sub_tree_root.inner().to_vec());
            lt_master_tree.insert_leaf(from_index, leaf);
        }
        let lt_root = lt_master_tree.root().map_err(Error::Merkle)?;
        info!(
            "LT_UPDATE, DOMAIN: {}, NEW_MERKLE_ROOT: {}",
            domain.to_hash(),
            lt_root,
        );

        Ok(())
    }

    pub fn update_trust_map(
        &mut self,
        domain: Domain,
        trust_entries: Vec<TrustEntry>,
    ) -> Result<(), Error> {
        let domain_indices = self
            .indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::IndicesNotFound(domain.to_hash()))?;
        let rev_domain_indices = self
            .rev_indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::ReverseIndicesNotFound(domain.to_hash()))?;
        let count = self
            .count
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::CountNotFound(domain.to_hash()))?;
        let lt = self
            .local_trust
            .get_mut(&domain.trust_namespace())
            .ok_or::<Error>(Error::LocalTrustNotFound(domain.trust_namespace()))?;
        for entry in trust_entries {
            let from_index = if let Some(i) = domain_indices.get(entry.from()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.from().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.from().clone());
                *count += 1;
                curr_count
            };
            let to_index = if let Some(i) = domain_indices.get(entry.to()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.to().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.to().clone());
                *count += 1;
                curr_count
            };

            let from_map = lt.entry(from_index).or_insert(OutboundLocalTrust::new());
            let is_zero = entry.value() == &0.0;
            let exists = from_map.contains_key(&to_index);
            if is_zero && exists {
                from_map.remove(&to_index);
            } else if !is_zero {
                from_map.insert(to_index, *entry.value());
            }
        }
        info!("LT_MAP_UPDATE, DOMAIN: {}", domain.to_hash(),);

        Ok(())
    }

    pub fn update_seed(
        &mut self,
        domain: Domain,
        seed_entries: Vec<ScoreEntry>,
    ) -> Result<(), Error> {
        let domain_indices = self
            .indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::IndicesNotFound(domain.to_hash()))?;
        let rev_domain_indices = self
            .rev_indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::ReverseIndicesNotFound(domain.to_hash()))?;
        let count = self
            .count
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::CountNotFound(domain.to_hash()))?;
        let st_master_tree = self
            .st_master_tree
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::SeedTrustMasterTreeNotFound(domain.to_hash()))?;
        let seed = self
            .seed_trust
            .get_mut(&domain.seed_namespace())
            .ok_or::<Error>(Error::SeedTrustNotFound(domain.seed_namespace()))?;
        for entry in seed_entries {
            let index = if let Some(i) = domain_indices.get(entry.id()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.id().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.id().clone());
                *count += 1;
                curr_count
            };
            let is_zero = entry.value() == &0.0;
            let exists = seed.contains_key(&index);
            if is_zero && exists {
                seed.remove(&index);
            } else if !is_zero {
                seed.insert(index, *entry.value());
            }

            let leaf = hash_leaf::<Keccak256>(entry.value().to_be_bytes().to_vec());
            st_master_tree.insert_leaf(index, leaf);
        }
        let st_root = st_master_tree.root().map_err(Error::Merkle)?;
        info!(
            "ST_UPDATE, DOMAIN: {}, NEW_MERKLE_ROOT: {}",
            domain.to_hash(),
            st_root,
        );

        Ok(())
    }

    pub fn update_seed_map(
        &mut self,
        domain: Domain,
        seed_entries: Vec<ScoreEntry>,
    ) -> Result<(), Error> {
        let domain_indices = self
            .indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::IndicesNotFound(domain.to_hash()))?;
        let rev_domain_indices = self
            .rev_indices
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::ReverseIndicesNotFound(domain.to_hash()))?;
        let count = self
            .count
            .get_mut(&domain.to_hash())
            .ok_or::<Error>(Error::CountNotFound(domain.to_hash()))?;
        let seed = self
            .seed_trust
            .get_mut(&domain.seed_namespace())
            .ok_or::<Error>(Error::SeedTrustNotFound(domain.seed_namespace()))?;
        for entry in seed_entries {
            let index = if let Some(i) = domain_indices.get(entry.id()) {
                *i
            } else {
                let curr_count = *count;
                domain_indices.insert(entry.id().clone(), curr_count);
                rev_domain_indices.insert(curr_count, entry.id().clone());
                *count += 1;
                curr_count
            };
            let is_zero = entry.value() == &0.0;
            let exists = seed.contains_key(&index);
            if is_zero && exists {
                seed.remove(&index);
            } else if !is_zero {
                seed.insert(index, *entry.value());
            }
        }
        info!("ST_MAP_UPDATE, DOMAIN: {}", domain.to_hash(),);

        Ok(())
    }

    pub fn get_base_root_hashes(&self, domain: &Domain) -> Result<Hash, Error> {
        let lt_tree = self
            .lt_master_tree
            .get(&domain.to_hash())
            .ok_or::<Error>(Error::LocalTrustMasterTreeNotFound(domain.to_hash()))?;
        let st_tree = self
            .st_master_tree
            .get(&domain.to_hash())
            .ok_or::<Error>(Error::SeedTrustMasterTreeNotFound(domain.to_hash()))?;
        let lt_tree_root = lt_tree.root().map_err(Error::Merkle)?;
        let st_tree_root = st_tree.root().map_err(Error::Merkle)?;
        let tree_roots = hash_two::<Keccak256>(lt_tree_root, st_tree_root);
        Ok(tree_roots)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("'indices' not found for domain: {0}")]
    IndicesNotFound(DomainHash),
    #[error("'rev_indices' not found for domain: {0}")]
    ReverseIndicesNotFound(DomainHash),
    #[error("'count' not found for domain: {0}")]
    CountNotFound(DomainHash),
    #[error("'local_trust_sub_trees' not found for domain: {0}")]
    LocalTrustSubTreesNotFoundWithDomain(DomainHash),
    #[error("'local_trust_sub_trees' not found for index: {0}")]
    LocalTrustSubTreesNotFoundWithIndex(u64),
    #[error("'local_trust_master_tree' not found for domain: {0}")]
    LocalTrustMasterTreeNotFound(DomainHash),
    #[error("'seed_trust_master_tree' not found for domain: {0}")]
    SeedTrustMasterTreeNotFound(DomainHash),
    #[error("'local_trust' not found for domain: {0}")]
    LocalTrustNotFound(OwnedNamespace),
    #[error("'seed_trust' not found for domain: {0}")]
    SeedTrustNotFound(OwnedNamespace),
    #[error("'domain_index' not found for address: {0}")]
    DomainIndexNotFound(String),
    #[error("Merkle Error: {0}")]
    Merkle(merkle::Error),
    #[error("Misc Error: {0}")]
    Misc(String),
}
