use crate::merkle::{self, hash_two, Hash};
use getset::Getters;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sha3::Digest;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
/// Dense Merkle tree.
/// The dense tree is a tree where leaf nodes are compressed to be next to each other
/// which makes it more efficient to store and traverse.
/// The tree is built from the fixed vector of leaves in the order they are given,
/// and cannot be modified after creation.
pub struct DenseMerkleTree<H>
where
    H: Digest,
{
    /// HashMap to keep the level and index of the nodes.
    nodes: HashMap<u8, Vec<Hash>>,
    // Number of levels
    num_levels: u8,
    /// Default hashes for each level (used for padding)
    defaults: Vec<Hash>,
    /// PhantomData for the hasher
    _h: PhantomData<H>,
}

impl<H> DenseMerkleTree<H>
where
    H: Digest,
{
    /// Returns the root of the tree.
    pub fn root(&self) -> Result<Hash, merkle::Error> {
        self.nodes
            .get(&self.num_levels)
            .map(|h| h[0].clone())
            .ok_or(merkle::Error::RootNotFound)
    }

    /// Generates a Merkle path (proof) for a given leaf at the specified index.
    ///
    /// The path contains the sibling hashes at each level needed to verify
    /// that the leaf is part of the tree. The path goes from the leaf level
    /// up to (but not including) the root.
    ///
    /// # Arguments
    /// * `index` - The index of the leaf in the tree (0-based)
    ///
    /// # Returns
    /// A vector of sibling hashes from leaf level to root level.
    pub fn generate_path(&self, index: usize) -> Result<Vec<Hash>, merkle::Error> {
        let leaves = self.nodes.get(&0).ok_or(merkle::Error::NodesNotFound)?;
        let padded_len = leaves.len();

        if index >= padded_len {
            return Err(merkle::Error::NodesNotFound);
        }

        let mut path = Vec::new();
        let mut current_index = index;

        for level in 0..self.num_levels {
            let level_nodes = self.nodes.get(&level).ok_or(merkle::Error::NodesNotFound)?;

            // Determine the sibling index
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            // Get the sibling hash (use level-appropriate default if out of bounds)
            let sibling_hash = if sibling_index < level_nodes.len() {
                level_nodes[sibling_index].clone()
            } else {
                self.defaults[level as usize].clone()
            };

            path.push(sibling_hash);

            // Move to the parent index for the next level
            current_index /= 2;
        }

        Ok(path)
    }

    /// Verifies a Merkle path (proof) for a given leaf hash and index against the expected root.
    ///
    /// This function takes a leaf hash, its index in the tree, and a path of sibling hashes,
    /// then reconstructs the root by hashing up the tree and compares it to the expected root.
    ///
    /// # Arguments
    /// * `leaf` - The hash of the leaf to verify
    /// * `index` - The index of the leaf in the tree (0-based)
    /// * `path` - The Merkle path (sibling hashes from leaf level to root level)
    /// * `expected_root` - The expected root hash to verify against
    ///
    /// # Returns
    /// `true` if the path is valid and leads to the expected root, `false` otherwise.
    pub fn verify_path(leaf: &Hash, index: usize, path: &[Hash], expected_root: &Hash) -> bool {
        let mut current = leaf.clone();
        let mut current_index = index;

        for sibling in path {
            if current_index % 2 == 0 {
                current = hash_two::<H>(current, sibling.clone());
            } else {
                current = hash_two::<H>(sibling.clone(), current);
            }
            current_index /= 2;
        }

        current == *expected_root
    }

    /// Builds a Merkle tree from the given leaf nodes.
    pub fn new(mut leaves: Vec<Hash>) -> Result<Self, merkle::Error> {
        let next_power_of_two = leaves.len().next_power_of_two();
        if leaves.len() < next_power_of_two {
            let diff = next_power_of_two - leaves.len();
            leaves.extend(vec![Hash::default(); diff]);
        }
        let num_levels = (u64::BITS - next_power_of_two.leading_zeros()) as u8;

        let mut defaults = Vec::new();
        defaults.push(Hash::default());
        for i in 1..num_levels as usize {
            let h = hash_two::<H>(defaults[i - 1].clone(), defaults[i - 1].clone());
            defaults.push(h);
        }

        let mut tree = HashMap::new();
        tree.insert(0u8, leaves);

        for i in 0..num_levels {
            let nodes = tree.get(&i).ok_or(merkle::Error::NodesNotFound)?;
            let next: Vec<Hash> = nodes
                .par_iter()
                .chunks(2)
                .map(|chunk| {
                    if chunk.len() == 2 {
                        hash_two::<H>(chunk[0].clone(), chunk[1].clone())
                    } else {
                        hash_two::<H>(chunk[0].clone(), defaults[i as usize].clone())
                    }
                })
                .collect();
            tree.insert(i + 1, next);
        }

        Ok(Self {
            nodes: tree,
            num_levels,
            defaults,
            _h: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::merkle::{fixed::DenseMerkleTree, Hash};
    use sha3::Keccak256;

    #[test]
    fn should_build_fixed_tree() {
        // Testing build_tree and find_path functions with arity 2
        let leaves = vec![
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
            Hash::default(),
        ];
        let merkle = DenseMerkleTree::<Keccak256>::new(leaves).unwrap();
        let root = merkle.root().unwrap();

        assert_eq!(
            root.to_hex(),
            "887c22bd8750d34016ac3c66b5ff102dacdd73f6b014e710b51e8022af9a1968".to_string()
        );
    }

    #[test]
    fn should_generate_valid_path() {
        use crate::merkle::hash_two;

        // Create a tree with 4 leaves
        let leaf0 = Hash::from_bytes([1u8; 32]);
        let leaf1 = Hash::from_bytes([2u8; 32]);
        let leaf2 = Hash::from_bytes([3u8; 32]);
        let leaf3 = Hash::from_bytes([4u8; 32]);

        let leaves = vec![leaf0.clone(), leaf1.clone(), leaf2.clone(), leaf3.clone()];
        let merkle = DenseMerkleTree::<Keccak256>::new(leaves).unwrap();

        // Generate path for leaf at index 0
        let path = merkle.generate_path(0).unwrap();
        // num_levels for 4 leaves = 3 (levels 0, 1, 2, with root at level 3)
        assert_eq!(path.len(), *merkle.num_levels() as usize);

        // Verify path for index 0:
        // Level 0: sibling is leaf1
        assert_eq!(path[0], leaf1);
        // Level 1: sibling is hash(leaf2, leaf3)
        let expected_sibling_1 = hash_two::<Keccak256>(leaf2.clone(), leaf3.clone());
        assert_eq!(path[1], expected_sibling_1);

        // Verify we can reconstruct the root using the path
        let mut current = leaf0.clone();
        let mut index = 0usize;
        for sibling in &path {
            if index % 2 == 0 {
                current = hash_two::<Keccak256>(current, sibling.clone());
            } else {
                current = hash_two::<Keccak256>(sibling.clone(), current);
            }
            index /= 2;
        }
        assert_eq!(current, merkle.root().unwrap());
    }

    #[test]
    fn should_verify_path() {
        // Create a tree with 4 leaves
        let leaf0 = Hash::from_bytes([1u8; 32]);
        let leaf1 = Hash::from_bytes([2u8; 32]);
        let leaf2 = Hash::from_bytes([3u8; 32]);
        let leaf3 = Hash::from_bytes([4u8; 32]);

        let leaves = vec![leaf0.clone(), leaf1.clone(), leaf2.clone(), leaf3.clone()];
        let merkle = DenseMerkleTree::<Keccak256>::new(leaves).unwrap();
        let root = merkle.root().unwrap();

        // Test verification for each leaf
        for i in 0..4 {
            let leaf = match i {
                0 => &leaf0,
                1 => &leaf1,
                2 => &leaf2,
                3 => &leaf3,
                _ => unreachable!(),
            };
            let path = merkle.generate_path(i).unwrap();
            assert!(
                DenseMerkleTree::<Keccak256>::verify_path(leaf, i, &path, &root),
                "Path verification failed for leaf at index {}",
                i
            );
        }

        // Test that verification fails with wrong leaf
        let wrong_leaf = Hash::from_bytes([99u8; 32]);
        let path = merkle.generate_path(0).unwrap();
        assert!(
            !DenseMerkleTree::<Keccak256>::verify_path(&wrong_leaf, 0, &path, &root),
            "Path verification should fail for wrong leaf"
        );

        // Test that verification fails with wrong index
        assert!(
            !DenseMerkleTree::<Keccak256>::verify_path(&leaf0, 1, &path, &root),
            "Path verification should fail for wrong index"
        );

        // Test that verification fails with wrong root
        let wrong_root = Hash::from_bytes([99u8; 32]);
        assert!(
            !DenseMerkleTree::<Keccak256>::verify_path(&leaf0, 0, &path, &wrong_root),
            "Path verification should fail for wrong root"
        );
    }
}
