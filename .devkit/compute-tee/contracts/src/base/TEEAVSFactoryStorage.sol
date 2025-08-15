// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IAllocationManager} from "eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IPermissionController} from "eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IKeyRegistrar} from "eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {ITEEAVSFactory} from "../interfaces/ITEEAVSFactory.sol";
import {ITEEOperator} from "../interfaces/ITEEOperator.sol";

abstract contract TEEAVSFactoryStorage is ITEEAVSFactory {
    /// @notice The EIP-712 typehash for the `AVSReleaseDeployed` struct used by the contract.
    bytes32 public constant AVS_RELEASE_DEPLOYED_TYPEHASH =
        keccak256("AVSReleaseDeployed(address avs,uint256 releaseId,bytes32 attestationHash)");

    /// @notice The operator set ID used by all AVSs deployed by this factory
    uint32 public constant OPERATOR_SET_ID = 0;

    /// @notice The TEEAVSRegistrar implementation to clone
    address public immutable avsImplementation;

    /// @notice The AllocationManager contract from EigenLayer
    IAllocationManager public immutable allocationManager;

    /// @notice The PermissionController contract from EigenLayer
    IPermissionController public immutable permissionController;

    /// @notice The KeyRegistrar contract from EigenLayer
    IKeyRegistrar public immutable keyRegistrar;

    /// @notice The TEEOperator contract from EigenLayer
    ITEEOperator public immutable teeOperator;

    constructor(
        address _avsImplementation,
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        IKeyRegistrar _keyRegistrar,
        ITEEOperator _teeOperator
    ) {
        avsImplementation = _avsImplementation;
        allocationManager = _allocationManager;
        permissionController = _permissionController;
        keyRegistrar = _keyRegistrar;
        teeOperator = _teeOperator;
    }
}
