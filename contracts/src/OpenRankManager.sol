// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {OpenRankManagerStorage} from "./OpenRankManagerStorage.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract OpenRankManager is OpenRankManagerStorage {
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    constructor() OpenRankManagerStorage() {}

    function allowlistUser(address user) onlyOwner external {
        allowlistedUsers[user] = true;
    }

    function allowlistComputer(address computer) onlyOwner external {
        allowlistedComputers[computer] = true;
    }

    function allowlistChallenger(address challenger) onlyOwner external {
        allowlistedChallengers[challenger] = true;
    }

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
        require(metaComputeRequests[computeId].id != 0, ComputeRequestNotFound());
        require(metaComputeResults[computeId].computeId == 0, ComputeResultAlreadySubmitted());

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
        require(metaComputeRequests[computeId].id != 0, ComputeRequestNotFound());
        require(metaComputeResults[computeId].computeId != 0, ComputeResultNotFound());

        uint256 computeDiff = block.timestamp - metaComputeResults[computeId].timestamp;
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

    function updateChallengeWindow(
        uint64 challengeWindow
    ) public onlyOwner {
        CHALLENGE_WINDOW = challengeWindow;
    }
}
