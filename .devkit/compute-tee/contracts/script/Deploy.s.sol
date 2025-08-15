// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {
    TransparentUpgradeableProxy,
    ITransparentUpgradeableProxy
} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import {IDelegationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IDelegationManager.sol";
import {IAllocationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IKeyRegistrar} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IReleaseManager} from "@eigenlayer-contracts/src/contracts/interfaces/IReleaseManager.sol";
import {EmptyContract} from "@eigenlayer-contracts/src/test/mocks/EmptyContract.sol";
import {ITEEAVSFactory} from "../src/interfaces/ITEEAVSFactory.sol";
import {ITEEOperator} from "../src/interfaces/ITEEOperator.sol";
import {TEEAVSFactory} from "../src/TEEAVSFactory.sol";
import {TEEAVSRegistrar} from "../src/TEEAVSRegistrar.sol";
import {TEEOperator} from "../src/TEEOperator.sol";

contract DeployScript is Script {
    struct DeployParams {
        string environment;
        string version;
        address delegationManager;
        address allocationManager;
        address permissionController;
        address keyRegistrar;
        address releaseManager;
        address proxyAdmin;
        address initialOwner;
        address delegationApprover;
        uint32 allocationDelay;
        string metadataURI;
    }

    struct DeployedContracts {
        address proxyAdmin;
        address teeAVSFactory;
        address teeAVSFactoryImpl;
        address teeAVSRegistrarImpl;
        address teeOperator;
        address teeOperatorImpl;
    }

    function run() public {
        string memory configPath = "script/config/deploy.json";
        string memory json = vm.readFile(configPath);

        DeployParams memory params = DeployParams({
            environment: vm.parseJsonString(json, ".environment"),
            version: vm.parseJsonString(json, ".version"),
            delegationManager: vm.parseJsonAddress(json, ".delegationManager"),
            allocationManager: vm.parseJsonAddress(json, ".allocationManager"),
            permissionController: vm.parseJsonAddress(json, ".permissionController"),
            keyRegistrar: vm.parseJsonAddress(json, ".keyRegistrar"),
            releaseManager: vm.parseJsonAddress(json, ".releaseManager"),
            proxyAdmin: vm.parseJsonAddress(json, ".proxyAdmin"),
            initialOwner: vm.parseJsonAddress(json, ".initialOwner"),
            delegationApprover: vm.parseJsonAddress(json, ".delegationApprover"),
            allocationDelay: uint32(vm.parseJsonUint(json, ".allocationDelay")),
            metadataURI: vm.parseJsonString(json, ".metadataURI")
        });

        run(params);
    }

    function run(DeployParams memory params) public {
        vm.startBroadcast();

        _writeOutputToJson(params.environment, deploy(params));

        vm.stopBroadcast();
    }

    function deploy(DeployParams memory params) public returns (DeployedContracts memory) {
        require(bytes(params.environment).length != 0, "Environment must not be empty");
        require(bytes(params.version).length != 0, "Version must not be empty");
        require(params.delegationManager != address(0), "Delegation manager must not be empty");
        require(params.allocationManager != address(0), "Allocation manager must not be empty");
        require(params.permissionController != address(0), "Permission controller must not be empty");
        require(params.keyRegistrar != address(0), "Key registrar must not be empty");
        require(params.releaseManager != address(0), "Release manager must not be empty");
        require(params.initialOwner != address(0), "Initial owner must not be empty");
        require(bytes(params.metadataURI).length != 0, "Metadata URI must not be empty");

        // Deploy proxy admin if not provided
        if (params.proxyAdmin == address(0)) {
            params.proxyAdmin = address(new ProxyAdmin());
        }
        EmptyContract emptyContract = new EmptyContract();

        // Deploy proxies
        TransparentUpgradeableProxy teeAVSFactoryProxy =
            new TransparentUpgradeableProxy(address(emptyContract), params.proxyAdmin, new bytes(0));
        TransparentUpgradeableProxy teeOperatorProxy =
            new TransparentUpgradeableProxy(address(emptyContract), params.proxyAdmin, new bytes(0));

        // Deploy implementation contracts
        TEEAVSRegistrar teeAVSRegistrarImpl = new TEEAVSRegistrar(
            IAllocationManager(params.allocationManager),
            IPermissionController(params.permissionController),
            IKeyRegistrar(params.keyRegistrar),
            IReleaseManager(params.releaseManager)
        );
        TEEAVSFactory teeAVSFactoryImpl = new TEEAVSFactory(
            params.version,
            address(teeAVSRegistrarImpl),
            IAllocationManager(params.allocationManager),
            IPermissionController(params.permissionController),
            IKeyRegistrar(params.keyRegistrar),
            ITEEOperator(address(teeOperatorProxy))
        );
        TEEOperator teeOperatorImpl = new TEEOperator(
            IDelegationManager(params.delegationManager),
            IAllocationManager(params.allocationManager),
            IPermissionController(params.permissionController),
            address(teeAVSFactoryProxy)
        );

        // Upgrade proxies using ProxyAdmin
        ProxyAdmin(params.proxyAdmin).upgrade(
            ITransparentUpgradeableProxy(address(teeAVSFactoryProxy)), address(teeAVSFactoryImpl)
        );
        ProxyAdmin(params.proxyAdmin).upgradeAndCall(
            ITransparentUpgradeableProxy(address(teeOperatorProxy)),
            address(teeOperatorImpl),
            abi.encodeCall(
                TEEOperator.initialize,
                (params.initialOwner, params.delegationApprover, params.allocationDelay, params.metadataURI)
            )
        );

        console.log("Proxy Admin:", address(params.proxyAdmin));
        console.log("TEEOperator deployed at:", address(teeOperatorProxy));
        console.log("TEEAVSFactory deployed at:", address(teeAVSFactoryProxy));
        console.log("TEEAVSRegistrar implementation deployed at:", address(teeAVSRegistrarImpl));

        return DeployedContracts({
            proxyAdmin: params.proxyAdmin,
            teeAVSFactory: address(teeAVSFactoryProxy),
            teeAVSFactoryImpl: address(teeAVSFactoryImpl),
            teeAVSRegistrarImpl: address(teeAVSRegistrarImpl),
            teeOperator: address(teeOperatorProxy),
            teeOperatorImpl: address(teeOperatorImpl)
        });
    }

    function _writeOutputToJson(string memory environment, DeployedContracts memory deployedContracts) internal {
        // Add the addresses object
        string memory addresses = "addresses";
        vm.serializeAddress(addresses, "proxyAdmin", deployedContracts.proxyAdmin);
        vm.serializeAddress(addresses, "teeAVSFactory", deployedContracts.teeAVSFactory);
        vm.serializeAddress(addresses, "teeAVSFactoryImpl", deployedContracts.teeAVSFactoryImpl);
        vm.serializeAddress(addresses, "teeAVSRegistrarImpl", deployedContracts.teeAVSRegistrarImpl);
        vm.serializeAddress(addresses, "teeOperator", deployedContracts.teeOperator);
        addresses = vm.serializeAddress(addresses, "teeOperatorImpl", deployedContracts.teeOperatorImpl);

        // Add the chainInfo object
        string memory chainInfo = "chainInfo";
        chainInfo = vm.serializeUint(chainInfo, "chainId", block.chainid);

        // Finalize the JSON
        string memory finalJson = "final";
        vm.serializeString(finalJson, "addresses", addresses);
        finalJson = vm.serializeString(finalJson, "chainInfo", chainInfo);

        // Write to output file
        string memory outputFile = string.concat("script/output/", environment, "/deploy_l1_output.json");
        
        // Create directory if it doesn't exist
        string memory outputDir = string.concat("script/output/", environment);
        vm.createDir(outputDir, true);
        
        vm.writeJson(finalJson, outputFile);
    }
}
