// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

// EigenLayer core contracts and interfaces
import {CoreDeployLib} from "../lib/eigenlayer-middleware/test/utils/CoreDeployLib.sol";

// Deploy script
import {DeployScript} from "../script/Deploy.s.sol";
import {IAllocationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IKeyRegistrar} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {IDelegationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IDelegationManager.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IReleaseManager} from "@eigenlayer-contracts/src/contracts/interfaces/IReleaseManager.sol";

// Project contracts
import {TEEAVSFactory} from "../src/TEEAVSFactory.sol";
import {TEEAVSRegistrar} from "../src/TEEAVSRegistrar.sol";
import {TEEOperator} from "../src/TEEOperator.sol";
import {ITEEAVSFactory} from "../src/interfaces/ITEEAVSFactory.sol";
import {ITEEOperator} from "../src/interfaces/ITEEOperator.sol";

// Mock contracts for testing
import {EmptyContract} from "@eigenlayer-contracts/src/test/mocks/EmptyContract.sol";
import {ETHPOSDepositMock} from "@eigenlayer-contracts/src/test/mocks/ETHDepositMock.sol";

contract TEEAVSFactoryTest is Test {
    using CoreDeployLib for *;

    // Core deployment data
    CoreDeployLib.DeploymentData public coreDeployment;

    // Contract instances
    TEEAVSFactory public teeAVSFactory;
    TEEOperator public teeOperator;
    address public testAVSRegistrar;

    // Deploy script instance
    DeployScript public deployScript;

    // Infrastructure
    ProxyAdmin public proxyAdmin;
    EmptyContract public emptyContract;
    ETHPOSDepositMock public ethPOSDepositMock;

    // Test addresses
    address public owner = address(0x1);
    address public pauser = address(0x2);
    address public unpauser = address(0x3);
    address public operatorAddress = address(0x4);
    address public testAVS = address(0x100);

    // Mock for components not yet deployed in CoreDeployLib
    address public mockKeyRegistrar = address(0x200);
    address public mockReleaseManager = address(0x201);

    function setUp() public {
        vm.startPrank(owner);

        // Deploy infrastructure
        proxyAdmin = new ProxyAdmin();
        emptyContract = new EmptyContract();
        ethPOSDepositMock = new ETHPOSDepositMock();

        // Configure and deploy EigenLayer core contracts
        CoreDeployLib.DeploymentConfigData memory config = _getDeploymentConfig();
        coreDeployment = CoreDeployLib.deployContracts(address(proxyAdmin), config);

        // Deploy contracts using DeployScript
        _deployContracts();

        // Label contracts for debugging
        _labelContracts();

        vm.stopPrank();
    }

    function test_createAVS() public {
        address admin = address(0x100);
        string memory metadataURI = "https://example.com/metadata";

        // Mock the KeyRegistrar.configureOperatorSet function
        vm.mockCall(mockKeyRegistrar, abi.encodeWithSelector(IKeyRegistrar.configureOperatorSet.selector), "");

        // Mock the ReleaseManager.publishMetadataURI function
        vm.mockCall(mockReleaseManager, abi.encodeWithSelector(IReleaseManager.publishMetadataURI.selector), "");

        vm.startPrank(admin);

        // Expect the AVSCreated event to be emitted (we don't check the avsAddress since it's computed)
        vm.expectEmit(true, true, false, true);
        emit ITEEAVSFactory.AVSCreated(admin, admin, address(0), metadataURI);

        // Execute the test - create an AVS
        address createdAVS = teeAVSFactory.createAVS(admin, metadataURI);

        // Verify the AVS was created and is not zero address
        assertTrue(createdAVS != address(0), "Created AVS should not be zero address");

        // Verify it's a clone by checking it has code
        assertTrue(createdAVS.code.length > 0, "Created AVS should have code");

        vm.stopPrank();
    }

    function test_createAVS_invalidAdmin() public {
        string memory metadataURI = "https://example.com/metadata";

        // Expect revert when admin is zero address
        vm.expectRevert(abi.encodeWithSelector(ITEEAVSFactory.InvalidAdmin.selector));
        teeAVSFactory.createAVS(address(0), metadataURI);
    }

    function test_createAVS_invalidMetadataURI() public {
        address admin = address(0x100);

        // Expect revert when metadata URI is empty
        vm.expectRevert(abi.encodeWithSelector(ITEEAVSFactory.InvalidMetadataURI.selector));
        teeAVSFactory.createAVS(admin, "");
    }

    function _getDeploymentConfig() internal view returns (CoreDeployLib.DeploymentConfigData memory) {
        return CoreDeployLib.DeploymentConfigData({
            strategyManager: CoreDeployLib.StrategyManagerConfig({
                initPausedStatus: 0,
                initialOwner: owner,
                initialStrategyWhitelister: owner
            }),
            delegationManager: CoreDeployLib.DelegationManagerConfig({
                initPausedStatus: 0,
                initialOwner: owner,
                minWithdrawalDelayBlocks: uint32(7 days / 12 seconds)
            }),
            eigenPodManager: CoreDeployLib.EigenPodManagerConfig({initPausedStatus: 0, initialOwner: owner}),
            allocationManager: CoreDeployLib.AllocationManagerConfig({
                initPausedStatus: 0,
                initialOwner: owner,
                deallocationDelay: uint32(7 days),
                allocationConfigurationDelay: uint32(1 days)
            }),
            strategyFactory: CoreDeployLib.StrategyFactoryConfig({initPausedStatus: 0, initialOwner: owner}),
            rewardsCoordinator: CoreDeployLib.RewardsCoordinatorConfig({
                initPausedStatus: 0,
                initialOwner: owner,
                rewardsUpdater: owner,
                activationDelay: uint32(7 days),
                defaultSplitBips: 10000,
                calculationIntervalSeconds: uint32(1 weeks),
                maxRewardsDuration: uint32(6 * 30 days),
                maxRetroactiveLength: uint32(84 days),
                maxFutureLength: uint32(30 days),
                genesisRewardsTimestamp: uint32((block.timestamp / (1 weeks)) * (1 weeks))
            }),
            avsDirectory: CoreDeployLib.AVSDirectoryConfig({initPausedStatus: 0, initialOwner: owner}),
            ethPOSDeposit: CoreDeployLib.ETHPOSDepositConfig({
                ethPOSDepositAddress: address(ethPOSDepositMock) // Use deployed mock
            }),
            eigenPod: CoreDeployLib.EigenPodConfig({genesisTimestamp: uint64(block.timestamp)})
        });
    }

    function _deployContracts() internal {
        // Create deploy script instance
        deployScript = new DeployScript();

        // Set up deployment parameters
        DeployScript.DeployParams memory params = DeployScript.DeployParams({
            version: "1.0.0",
            environment: "test",
            delegationManager: coreDeployment.delegationManager,
            allocationManager: coreDeployment.allocationManager,
            permissionController: coreDeployment.permissionController,
            keyRegistrar: mockKeyRegistrar,
            releaseManager: mockReleaseManager,
            proxyAdmin: address(0),
            initialOwner: owner,
            delegationApprover: owner,
            allocationDelay: uint32(7 days),
            metadataURI: "https://example.com/metadata"
        });

        // Deploy contracts using the script
        DeployScript.DeployedContracts memory deployed = deployScript.deploy(params);

        // Set contract instances from deployed addresses
        teeOperator = TEEOperator(payable(deployed.teeOperator));
        teeAVSFactory = TEEAVSFactory(deployed.teeAVSFactory);
    }

    function _labelContracts() internal {
        vm.label(address(proxyAdmin), "ProxyAdmin");
        vm.label(address(emptyContract), "EmptyContract");
        vm.label(address(ethPOSDepositMock), "ETHPOSDepositMock");
        vm.label(address(teeAVSFactory), "TEEAVSFactory");
        vm.label(address(teeOperator), "TEEOperator");
        vm.label(coreDeployment.delegationManager, "DelegationManager");
        vm.label(coreDeployment.strategyManager, "StrategyManager");
        vm.label(coreDeployment.avsDirectory, "AVSDirectory");
        vm.label(coreDeployment.allocationManager, "AllocationManager");
        vm.label(coreDeployment.rewardsCoordinator, "RewardsCoordinator");
        vm.label(coreDeployment.permissionController, "PermissionController");
        vm.label(mockKeyRegistrar, "MockKeyRegistrar");
        vm.label(mockReleaseManager, "MockReleaseManager");
        vm.label(testAVS, "TestAVS");
        vm.label(owner, "Owner");
        vm.label(operatorAddress, "OperatorAddress");
    }
}
