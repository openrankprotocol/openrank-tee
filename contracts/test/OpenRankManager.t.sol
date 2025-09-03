// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {OpenRankManager} from "../src/OpenRankManager.sol";
import {DeployOpenRank} from "../script/DeployOpenRank.s.sol";

contract OpenRankManagerTest is Test {
    OpenRankManager orManager;

    function setUp() public {
        address proxy = Upgrades.deployUUPSProxy(
            "OpenRankManager.sol:OpenRankManager",
            abi.encodeCall(OpenRankManager.initialize,())
        );

        orManager = OpenRankManager(proxy);
    }

    function testMetaChallenge() public {
        uint256 jobId = orManager.submitMetaComputeRequest(
            bytes32(0)
        );
        orManager.submitMetaComputeResult(
            jobId,
            bytes32(0),
            bytes32(0)
        );
    }
}
