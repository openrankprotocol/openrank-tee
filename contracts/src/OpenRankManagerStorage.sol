// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IOpenRankManager} from "./IOpenRankManager.sol";

abstract contract OpenRankManagerStorage is IOpenRankManager {
    uint64 public CHALLENGE_WINDOW = 60 * 60; // 60 minutes

    address owner;

    uint256 public idCounter;

    mapping(address => bool) allowlistedComputers;
    mapping(address => bool) allowlistedChallengers;
    mapping(address => bool) allowlistedUsers;

    mapping(uint256 => MetaComputeRequest) public metaComputeRequests;
    mapping(uint256 => MetaComputeResult) public metaComputeResults;
    mapping(uint256 => MetaChallenge) public metaChallenges;

    constructor() {
        idCounter = 1;

        allowlistedComputers[msg.sender] = true;
        allowlistedChallengers[msg.sender] = true;
        allowlistedUsers[msg.sender] = true;

        owner = msg.sender;
    }
}
