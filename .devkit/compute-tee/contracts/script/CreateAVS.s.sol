// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {stdJson} from "forge-std/StdJson.sol";
import {Script, console} from "forge-std/Script.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {ITEEAVSFactory} from "../src/interfaces/ITEEAVSFactory.sol";

contract CreateAVSScript is Script {
    using stdJson for string;

    function run(
        string memory environment,
        string memory metadataURI,
        address permissionController,
        address teeAVSFactory
    ) public {
        // Load the private keys from the environment variables
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY_DEPLOYER");
        address deployer = vm.addr(deployerPrivateKey);

        uint256 avsAdminPrivateKey = vm.envOr("PRIVATE_KEY_AVS_ADMIN", deployerPrivateKey);
        address avsAdmin = vm.addr(avsAdminPrivateKey);

        vm.startBroadcast(deployerPrivateKey);
        console.log("[CreateAVS] Using deployer address:", deployer);

        ITEEAVSFactory factory = ITEEAVSFactory(teeAVSFactory);

        console.log("[CreateAVS] Creating AVS via TEEAVSFactory contract:", teeAVSFactory);

        // Deploy a new TEE AVS instance via factory: creates a cloned AVS registrar,
        // initializes it with admin/metadata, and automatically registers the TEE operator.
        address avs = factory.createAVS(avsAdmin, metadataURI);
        console.log("[CreateAVS] AVS deployment completed successfully! Contract address:", avs);

        // Accept admin role for AVS
        if (avsAdmin != deployer) {
            vm.stopBroadcast();
            vm.startBroadcast(avsAdminPrivateKey);
        }
        IPermissionController(permissionController).acceptAdmin(avs);
        console.log("[CreateAVS] Admin role accepted by:", avsAdmin, "for AVS:", avs);

        vm.stopBroadcast();

        // Write deployment info to output file
        _writeOutputToJson(environment, avs);
    }

    function _writeOutputToJson(string memory environment, address teeAVSRegistrar) internal {
        // Add the addresses object
        string memory addresses = "addresses";
        addresses = vm.serializeAddress(addresses, "teeAVSRegistrar", teeAVSRegistrar);

        // Add the chainInfo object
        string memory chainInfo = "chainInfo";
        chainInfo = vm.serializeUint(chainInfo, "chainId", block.chainid);

        // Finalize the JSON
        string memory finalJson = "final";
        vm.serializeString(finalJson, "addresses", addresses);
        finalJson = vm.serializeString(finalJson, "chainInfo", chainInfo);

        // Write to output file
        string memory outputFile = string.concat("script/output/", environment, "/deploy_avs_l1_output.json");
        
        // Create directory if it doesn't exist
        string memory outputDir = string.concat("script/output/", environment);
        vm.createDir(outputDir, true);
        
        vm.writeJson(finalJson, outputFile);
    }
}
