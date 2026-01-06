// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {OpenRankManagerStorage} from "./OpenRankManagerStorage.sol";
import {MerklePathVerifier} from "./MerklePathVerifier.sol";

contract OpenRankManager is OpenRankManagerStorage {
    function initialize() public initializer {
        idCounter = 1;
        CHALLENGE_WINDOW = 60 * 60; // 60 minutes

        allowlistedComputers[msg.sender] = true;
        allowlistedChallengers[msg.sender] = true;
        allowlistedUsers[msg.sender] = true;

        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
    }

    function _authorizeUpgrade(address) internal override onlyOwner {}

    // ---------------------------------------------------------------
    // Meta Jobs
    // ---------------------------------------------------------------

    function submitMetaComputeRequest(
        bytes32 jobDescriptionId
    ) external returns (uint256 computeId) {
        require(allowlistedUsers[msg.sender], CallerNotWhitelisted());
        MetaComputeRequest memory computeRequest = MetaComputeRequest({
            user: msg.sender,
            id: idCounter,
            jobDescriptionId: jobDescriptionId,
            timestamp: block.timestamp
        });
        metaComputeRequests[idCounter] = computeRequest;

        emit MetaComputeRequestEvent(idCounter, jobDescriptionId);

        computeId = idCounter;
        idCounter += 1;
    }

    function submitMetaComputeResult(
        uint256 computeId,
        bytes32 metaCommitment,
        bytes32 resultsId
    ) external returns (bool) {
        require(allowlistedComputers[msg.sender], CallerNotWhitelisted());
        require(
            metaComputeRequests[computeId].id != 0,
            ComputeRequestNotFound()
        );
        require(
            metaComputeResults[computeId].computeId == 0,
            ComputeResultAlreadySubmitted()
        );

        MetaComputeResult memory computeResult = MetaComputeResult({
            computer: msg.sender,
            computeId: computeId,
            metaCommitment: metaCommitment,
            resultsId: resultsId,
            timestamp: block.timestamp
        });
        metaComputeResults[computeId] = computeResult;

        emit MetaComputeResultEvent(computeId, metaCommitment, resultsId);

        return true;
    }

    function submitMetaChallenge(
        uint256 computeId,
        uint32 subJobId
    ) external returns (bool) {
        require(allowlistedChallengers[msg.sender], CallerNotWhitelisted());
        require(
            metaComputeRequests[computeId].id != 0,
            ComputeRequestNotFound()
        );
        require(
            metaComputeResults[computeId].computeId != 0,
            ComputeResultNotFound()
        );

        uint256 computeDiff = block.timestamp -
            metaComputeResults[computeId].timestamp;
        require(computeDiff <= CHALLENGE_WINDOW, ChallengePeriodExpired());

        MetaChallenge memory challenge = MetaChallenge({
            challenger: msg.sender,
            computeId: computeId,
            subJobId: subJobId,
            timestamp: block.timestamp
        });
        metaChallenges[computeId] = challenge;

        emit MetaChallengeEvent(computeId, subJobId);
        return true;
    }

    // ---------------------------------------------------------------
    // Verification
    // ---------------------------------------------------------------

    /// @notice Verifies a score inclusion proof from the score-proof server
    /// @param computeId The compute ID to verify against
    /// @param scoreBytes The score value as bytes4 (f32 in big-endian format)
    /// @param scoreIndex The index of the score in the scores tree
    /// @param scoresTreePath The Merkle path for the score in the scores tree
    /// @param scoresTreeRoot The root of the scores tree (job commitment)
    /// @param metaIndex The index of the job commitment in the meta tree
    /// @param metaTreePath The Merkle path for the commitment in the meta tree
    /// @return True if the proof is valid and matches the on-chain commitment
    function verifyScoreProof(
        uint256 computeId,
        bytes4 scoreBytes,
        uint256 scoreIndex,
        bytes32[] calldata scoresTreePath,
        bytes32 scoresTreeRoot,
        uint256 metaIndex,
        bytes32[] calldata metaTreePath
    ) external view returns (bool) {
        require(
            metaComputeResults[computeId].computeId != 0,
            ComputeResultNotFound()
        );

        bytes32 metaTreeRoot = metaComputeResults[computeId].metaCommitment;

        // Verify score is included in the scores tree
        bytes32 scoreLeaf = MerklePathVerifier.hashScoreLeaf(scoreBytes);
        if (
            !MerklePathVerifier.verifyPath(
                scoreLeaf,
                scoreIndex,
                scoresTreePath,
                scoresTreeRoot
            )
        ) {
            return false;
        }

        // Verify scores tree root (commitment) is included in the meta tree
        if (
            !MerklePathVerifier.verifyPath(
                scoresTreeRoot,
                metaIndex,
                metaTreePath,
                metaTreeRoot
            )
        ) {
            return false;
        }

        return true;
    }

    // ---------------------------------------------------------------
    // Getters
    // ---------------------------------------------------------------

    function isAllowlistedComputer(
        address computer
    ) public view returns (bool) {
        return allowlistedComputers[computer];
    }

    // ---------------------------------------------------------------
    // Setters
    // ---------------------------------------------------------------

    function updateChallengeWindow(uint64 challengeWindow) public onlyOwner {
        CHALLENGE_WINDOW = challengeWindow;
    }

    function allowlistUser(address user) external onlyOwner {
        allowlistedUsers[user] = true;
    }

    function allowlistComputer(address computer) external onlyOwner {
        allowlistedComputers[computer] = true;
    }

    function allowlistChallenger(address challenger) external onlyOwner {
        allowlistedChallengers[challenger] = true;
    }
}
