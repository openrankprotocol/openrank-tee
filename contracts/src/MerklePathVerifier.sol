// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MerklePathVerifier
/// @notice Library for verifying Merkle proofs returned by the OpenRank score-proof server
/// @dev Uses keccak256 for hashing, matching the Rust implementation using sha3::Keccak256
library MerklePathVerifier {
    /// @notice Verifies a Merkle path (proof) for a given leaf hash
    /// @param leaf The hash of the leaf to verify
    /// @param index The index of the leaf in the tree (0-based)
    /// @param path The Merkle path (sibling hashes from leaf level to root level)
    /// @param expectedRoot The expected root hash to verify against
    /// @return True if the path is valid and leads to the expected root
    function verifyPath(
        bytes32 leaf,
        uint256 index,
        bytes32[] calldata path,
        bytes32 expectedRoot
    ) internal pure returns (bool) {
        bytes32 current = leaf;
        uint256 currentIndex = index;

        for (uint256 i = 0; i < path.length; i++) {
            bytes32 sibling = path[i];

            if (currentIndex % 2 == 0) {
                // Current node is on the left, sibling is on the right
                current = keccak256(abi.encodePacked(current, sibling));
            } else {
                // Current node is on the right, sibling is on the left
                current = keccak256(abi.encodePacked(sibling, current));
            }

            currentIndex = currentIndex / 2;
        }

        return current == expectedRoot;
    }

    /// @notice Hashes a score value to create a leaf hash
    /// @dev Matches the Rust implementation: hash_leaf(score.to_be_bytes())
    /// @param score The score value as bytes4 (f32 in big-endian format)
    /// @return The keccak256 hash of the score bytes
    function hashScoreLeaf(bytes4 score) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(score));
    }

    /// @notice Verifies a complete score inclusion proof
    /// @dev Verifies both the scores tree path and the meta tree path
    /// @param scoreBytes The score value as bytes4 (f32 in big-endian format)
    /// @param scoreIndex The index of the score in the scores tree
    /// @param scoresTreePath The Merkle path for the score in the scores tree
    /// @param scoresTreeRoot The root of the scores tree (should match the job commitment)
    /// @param metaIndex The index of the job commitment in the meta tree
    /// @param metaTreePath The Merkle path for the commitment in the meta tree
    /// @param metaTreeRoot The root of the meta tree (the final on-chain commitment)
    /// @return True if both proofs are valid
    function verifyScoreProof(
        bytes4 scoreBytes,
        uint256 scoreIndex,
        bytes32[] calldata scoresTreePath,
        bytes32 scoresTreeRoot,
        uint256 metaIndex,
        bytes32[] calldata metaTreePath,
        bytes32 metaTreeRoot
    ) internal pure returns (bool) {
        // First, verify the score is included in the scores tree
        bytes32 scoreLeaf = hashScoreLeaf(scoreBytes);
        if (!verifyPath(scoreLeaf, scoreIndex, scoresTreePath, scoresTreeRoot)) {
            return false;
        }

        // Then, verify the scores tree root (commitment) is included in the meta tree
        if (!verifyPath(scoresTreeRoot, metaIndex, metaTreePath, metaTreeRoot)) {
            return false;
        }

        return true;
    }
}
