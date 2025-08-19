use crate::{
    algos::et::positive_run,
    merkle::{self, fixed::DenseMerkleTree, hash_leaf, Hash},
    tx::trust::{ScoreEntry, TrustEntry},
    Domain, DomainHash,
};
use getset::Getters;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha3::Keccak256;
use std::collections::HashMap;
use tracing::info;

use super::{BaseRunner, Error as BaseError};

#[derive(Getters)]
#[getset(get = "pub")]
/// Struct containing the state of the computer compute runner.
pub struct ComputeRunner {
    base: BaseRunner,
    compute_results: HashMap<DomainHash, Vec<(u64, f32)>>,
    compute_tree: HashMap<DomainHash, DenseMerkleTree<Keccak256>>,
}

impl ComputeRunner {
    pub fn new(domains: &[Domain]) -> Self {
        let base = BaseRunner::new(domains);
        let mut compute_results = HashMap::new();
        for domain in domains {
            let domain_hash = domain.to_hash();
            compute_results.insert(domain_hash, Vec::<(u64, f32)>::new());
        }
        Self {
            base,
            compute_results,
            compute_tree: HashMap::new(),
        }
    }

    /// Update the state of trees for certain domain, with the given trust entries.
    pub fn update_trust(
        &mut self,
        domain: Domain,
        trust_entries: Vec<TrustEntry>,
    ) -> Result<(), Error> {
        self.base
            .update_trust(domain, trust_entries)
            .map_err(Error::Base)
    }

    pub fn update_trust_map(
        &mut self,
        domain: Domain,
        trust_entries: Vec<TrustEntry>,
    ) -> Result<(), Error> {
        self.base
            .update_trust_map(domain, trust_entries)
            .map_err(Error::Base)
    }

    /// Update the state of trees for certain domain, with the given seed entries.
    pub fn update_seed(
        &mut self,
        domain: Domain,
        seed_entries: Vec<ScoreEntry>,
    ) -> Result<(), Error> {
        self.base
            .update_seed(domain, seed_entries)
            .map_err(Error::Base)
    }

    pub fn update_seed_map(
        &mut self,
        domain: Domain,
        seed_entries: Vec<ScoreEntry>,
    ) -> Result<(), Error> {
        self.base
            .update_seed_map(domain, seed_entries)
            .map_err(Error::Base)
    }

    /// Compute the EigenTrust scores for certain domain.
    pub fn compute(&mut self, domain: Domain) -> Result<(), Error> {
        info!("COMPUTE_RUN: {}", domain.to_hash());
        let lt = self
            .base
            .local_trust
            .get(&domain.trust_namespace())
            .ok_or::<Error>(BaseError::LocalTrustNotFound(domain.trust_namespace()).into())?;
        let seed = self
            .base
            .seed_trust
            .get(&domain.seed_namespace())
            .ok_or::<Error>(BaseError::SeedTrustNotFound(domain.seed_namespace()).into())?;
        let count = self
            .base
            .count
            .get(&domain.to_hash())
            .ok_or::<Error>(BaseError::CountNotFound(domain.to_hash()).into())?;
        let res = positive_run(lt.clone(), seed.clone(), *count);
        self.compute_results.insert(domain.to_hash(), res);
        Ok(())
    }

    /// Create the compute tree for certain domain.
    pub fn create_compute_tree(&mut self, domain: Domain) -> Result<(), Error> {
        info!("CREATE_COMPUTE_TREE: {}", domain.to_hash());
        let scores = self
            .compute_results
            .get(&domain.to_hash())
            .ok_or(Error::ComputeResultsNotFound(domain.to_hash()))?;
        let score_hashes: Vec<Hash> = scores
            .par_iter()
            .map(|(_, x)| hash_leaf::<Keccak256>(x.to_be_bytes().to_vec()))
            .collect();
        let compute_tree =
            DenseMerkleTree::<Keccak256>::new(score_hashes).map_err(Error::Merkle)?;
        info!(
            "COMPUTE_TREE_ROOT_HASH: {}",
            compute_tree.root().map_err(Error::Merkle)?
        );
        self.compute_tree.insert(domain.to_hash(), compute_tree);
        Ok(())
    }

    /// Get the compute scores for certain domain.
    pub fn get_compute_scores(&self, domain: Domain) -> Result<Vec<ScoreEntry>, Error> {
        let domain_indices = self
            .base
            .indices
            .get(&domain.to_hash())
            .ok_or::<Error>(BaseError::IndicesNotFound(domain.to_hash()).into())?;
        let scores = self
            .compute_results
            .get(&domain.to_hash())
            .ok_or(Error::ComputeResultsNotFound(domain.to_hash()))?;
        let index_to_address: HashMap<&u64, &String> =
            domain_indices.iter().map(|(k, v)| (v, k)).collect();

        let mut entries = Vec::new();
        for (index, val) in scores {
            let address = index_to_address
                .get(&index)
                .ok_or(Error::IndexToAddressNotFound(*index))?;
            let score_entry = ScoreEntry::new((*address).clone(), *val);
            entries.push(score_entry);
        }
        Ok(entries)
    }

    /// Get the local trust root hash and compute tree root hash for certain domain.
    pub fn get_root_hashes(&self, domain: Domain) -> Result<(Hash, Hash), Error> {
        let tree_roots = self.base.get_base_root_hashes(&domain)?;

        let compute_tree = self
            .compute_tree
            .get(&domain.to_hash())
            .ok_or(Error::ComputeTreeNotFound(domain.to_hash()))?;
        let ct_tree_root = compute_tree.root().map_err(Error::Merkle)?;

        Ok((tree_roots, ct_tree_root))
    }
}

#[derive(thiserror::Error, Debug)]
/// Errors that can arise while using the compute runner.
pub enum Error {
    #[error("Base Error: {0}")]
    Base(BaseError),
    /// The compute results for the domain are not found.
    #[error("ComputeResultsNotFound Error: {0}")]
    ComputeResultsNotFound(DomainHash),
    /// The index to address mapping for the domain are not found.
    #[error("IndexToAddressNotFound Error: {0}")]
    IndexToAddressNotFound(u64),
    /// The compute tree for the domain are not found.
    #[error("ComputeTreeNotFound Error: {0}")]
    ComputeTreeNotFound(DomainHash),
    /// The compute merkle tree error.
    #[error("Merkle Error: {0}")]
    Merkle(merkle::Error),
}

impl From<BaseError> for Error {
    fn from(err: BaseError) -> Self {
        Self::Base(err)
    }
}
