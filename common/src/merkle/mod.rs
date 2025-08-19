use crate::format_hex;
use alloy::hex;
use alloy_rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(test)]
use rand::Rng;

pub mod fixed;
pub mod incremental;

#[derive(
    Debug, Clone, Hash, Default, PartialEq, Eq, RlpDecodable, RlpEncodable, Serialize, Deserialize,
)]
/// Used to represent a hash of a node in the merkle tree.
pub struct Hash(#[serde(with = "hex")] [u8; 32]);

impl Hash {
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0; 32];
        if slice.len() > 32 {
            bytes.copy_from_slice(&slice[..32]);
        } else {
            bytes[..slice.len()].copy_from_slice(&slice);
        }
        Self(<[u8; 32]>::try_from(bytes).unwrap())
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Converts the hash to a hex string.
    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }

    pub fn inner(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", format_hex(self.clone().to_hex()))
    }
}

#[cfg(test)]
impl Hash {
    /// Generates a random hash. This is used for testing.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Hash(rng.gen::<[u8; 32]>())
    }
}

/// Converts given index to the next index.
fn next_index(i: u64) -> u64 {
    if i % 2 == 1 {
        (i - 1) / 2
    } else {
        i / 2
    }
}

/// Converts given bytes to the bits.
pub fn to_bits(num: &[u8]) -> Vec<bool> {
    let len = num.len() * 8;
    let mut bits = Vec::new();
    for i in 0..len {
        let bit = num[i / 8] & (1 << (i % 8)) != 0;
        bits.push(bit);
    }
    bits
}

/// Converts given field element to the bits.
pub fn num_to_bits_vec(num: u64) -> Vec<bool> {
    let bits = to_bits(&num.to_le_bytes());

    bits[..u64::BITS as usize].to_vec()
}

/// Computes the hash from two hashes.
pub fn hash_two<H: Digest>(left: Hash, right: Hash) -> Hash {
    let mut hasher = H::new();
    hasher.update(left.0);
    hasher.update(right.0);
    let hash = hasher.finalize().to_vec();
    let mut bytes: [u8; 32] = [0; 32];
    bytes.copy_from_slice(&hash);
    Hash(bytes)
}

/// Hashes the given data(`Vec<u8>`).
pub fn hash_leaf<H: Digest>(preimage: Vec<u8>) -> Hash {
    let mut hasher = H::new();
    hasher.update(preimage);
    let hash = hasher.finalize().to_vec();
    let mut bytes: [u8; 32] = [0; 32];
    bytes.copy_from_slice(&hash);
    Hash(bytes)
}

#[derive(thiserror::Error, Debug)]
/// An error type for the merkle tree.
pub enum Error {
    /// The root of the merkle tree is not found.
    #[error("Root not found")]
    RootNotFound,
    /// The nodes are not found in the merkle tree.
    #[error("Nodes not found")]
    NodesNotFound,
}
