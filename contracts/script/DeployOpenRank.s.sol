// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {OpenRankManager} from "../src/OpenRankManager.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Vm} from "forge-std/Vm.sol";

contract DeployOpenRank is Script {
    OpenRankManager public orManager;

    address initialOwner;

    function run() public {
        initialOwner = msg.sender;

        vm.startBroadcast(initialOwner);
        orManager = new OpenRankManager();
        vm.stopBroadcast();
    }

    function testRun(address sender) public {
        vm.startPrank(sender);
        orManager = new OpenRankManager();
        vm.stopPrank();
    }
}
