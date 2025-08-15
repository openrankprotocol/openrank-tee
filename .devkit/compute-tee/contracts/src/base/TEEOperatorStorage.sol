// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IDelegationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IDelegationManager.sol";
import {
    IAllocationManager,
    IAllocationManagerTypes
} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IKeyRegistrar, IKeyRegistrarTypes} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IReleaseManager} from "@eigenlayer-contracts/src/contracts/interfaces/IReleaseManager.sol";
import {OperatorSet} from "@eigenlayer-contracts/src/contracts/libraries/OperatorSetLib.sol";
import {IStrategy} from "@eigenlayer-contracts/src/contracts/interfaces/IStrategy.sol";
import {ITEEOperator} from "../interfaces/ITEEOperator.sol";

abstract contract TEEOperatorStorage is ITEEOperator {
    /// @notice The EigenLayer DelegationManager contract
    IDelegationManager public immutable delegationManager;

    /// @notice The EigenLayer AllocationManager contract
    IAllocationManager public immutable allocationManager;

    /// @notice The EigenLayer PermissionController contract
    IPermissionController public immutable permissionController;

    /// @notice The AVS Factory contract
    address public immutable avsFactory;

    constructor(
        IDelegationManager _delegationManager,
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        address _avsFactory
    ) {
        delegationManager = _delegationManager;
        allocationManager = _allocationManager;
        permissionController = _permissionController;
        avsFactory = _avsFactory;
    }

    /**
     * @dev This empty reserved space is put in place to allow future versions to add new
     * variables without shifting down storage in the inheritance chain.
     * See https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
     */
    uint256[50] private __gap;
}
