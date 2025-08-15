// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import { Script, console } from "forge-std/Script.sol";
import { stdJson } from "forge-std/StdJson.sol";

import { RandomnessBeacon } from "../src/RandomnessBeacon.sol";

contract DeployRandomnessBeacon is Script {
    using stdJson for string;

    struct Output {
        string name;
        address contractAddress;
    }

    function run(address owner) public {
        vm.startBroadcast();

        RandomnessBeacon randomnessBeacon = new RandomnessBeacon(owner);
        console.log("RandomnessBeacon deployed to:", address(randomnessBeacon));

        vm.stopBroadcast();

        // Write to output file
        Output[] memory outputs = new Output[](1);
        outputs[0] = Output({ name: "RandomnessBeacon", contractAddress: address(randomnessBeacon) });
        _writeOutputToJson(outputs);
    }

    function _writeOutputToJson(Output[] memory outputs) internal {
        uint256 length = outputs.length;

        if (length > 0) {
            // Add the addresses object
            string memory addresses = "addresses";

            for (uint256 i = 0; i < outputs.length - 1; i++) {
                vm.serializeAddress(addresses, outputs[i].name, outputs[i].contractAddress);
            }
            addresses = vm.serializeAddress(
                addresses, outputs[length - 1].name, outputs[length - 1].contractAddress
            );

            // Add the chainInfo object
            string memory chainInfo = "chainInfo";
            chainInfo = vm.serializeUint(chainInfo, "chainId", block.chainid);

            // Finalize the JSON
            string memory finalJson = "final";
            vm.serializeString(finalJson, "addresses", addresses);
            finalJson = vm.serializeString(finalJson, "chainInfo", chainInfo);

            // Write to output file
            string memory outputFile = "outputs/testnet/deploy_randomness_beacon_output.json";
            vm.writeJson(finalJson, outputFile);
        }
    }
}