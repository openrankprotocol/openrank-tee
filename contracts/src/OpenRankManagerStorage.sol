// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IOpenRankManager} from "./IOpenRankManager.sol";
import { Initializable } from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import { UUPSUpgradeable } from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import { OwnableUpgradeable } from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

abstract contract OpenRankManagerStorage is IOpenRankManager, Initializable, UUPSUpgradeable, OwnableUpgradeable {
    uint64 public CHALLENGE_WINDOW;

    uint256 public idCounter;

    mapping(address => bool) allowlistedComputers;
    mapping(address => bool) allowlistedChallengers;
    mapping(address => bool) allowlistedUsers;

    mapping(uint256 => MetaComputeRequest) public metaComputeRequests;
    mapping(uint256 => MetaComputeResult) public metaComputeResults;
    mapping(uint256 => MetaChallenge) public metaChallenges;

    // Mind the gap
    uint256[49] __gap;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }
}
