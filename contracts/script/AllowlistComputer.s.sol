// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {OpenRankManager} from "../src/OpenRankManager.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { Upgrades, Options } from "@openzeppelin-foundry-upgrades/Upgrades.sol";
import {Vm} from "forge-std/Vm.sol";

contract AllowlistComputer is Script {
    function run() public {
        address computerAddress = vm.envAddress("TEE_ADDRESS");
        address orAddress = vm.envAddress("OPENRANK_MANAGER_ADDRESS");
        console.log("OR Address: ", orAddress);
        console.log("Computer Address: ", computerAddress);

        address initialOwner = msg.sender;
        vm.startBroadcast(initialOwner);
        OpenRankManager orManager = OpenRankManager(orAddress);
        orManager.allowlistComputer(computerAddress);
        vm.stopBroadcast();
    }
}
