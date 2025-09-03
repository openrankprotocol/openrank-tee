// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {OpenRankManager} from "../src/OpenRankManager.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { Upgrades, Options } from "@openzeppelin-foundry-upgrades/Upgrades.sol";
import {Vm} from "forge-std/Vm.sol";

contract DeployOpenRank is Script {
    function run() public {
        address initialOwner = msg.sender;
        address computerAddress = vm.envAddress("TEE_ADDRESS");
        vm.startBroadcast(initialOwner);
        address proxy = Upgrades.deployUUPSProxy(
            "OpenRankManager.sol:OpenRankManager",
            abi.encodeCall(OpenRankManager.initialize,())
        );

        OpenRankManager orManager = OpenRankManager(proxy);
        orManager.allowlistComputer(computerAddress);
        console.log("OR Address: ", address(orManager));
        vm.stopBroadcast();
    }
}
