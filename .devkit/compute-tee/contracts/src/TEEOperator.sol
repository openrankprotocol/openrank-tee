// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Initializable} from "@openzeppelin-upgrades/contracts/proxy/utils/Initializable.sol";
import {IDelegationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IDelegationManager.sol";
import {
    IAllocationManager,
    IAllocationManagerTypes
} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IKeyRegistrar, IKeyRegistrarTypes} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {OperatorSet} from "@eigenlayer-contracts/src/contracts/libraries/OperatorSetLib.sol";
import {TEEOperatorStorage} from "./base/TEEOperatorStorage.sol";
import {ITEEOperator} from "./interfaces/ITEEOperator.sol";

contract TEEOperator is Initializable, TEEOperatorStorage {
    modifier onlyAVSFactory() {
        require(msg.sender == avsFactory, NotAVSFactory());
        _;
    }

    constructor(
        IDelegationManager _delegationManager,
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        address _avsFactory
    ) TEEOperatorStorage(_delegationManager, _allocationManager, _permissionController, _avsFactory) {
        _disableInitializers();
    }

    function initialize(address admin, address delegationApprover, uint32 allocationDelay, string calldata _metadataURI)
        external
        initializer
    {
        // Register as an EigenLayer operator
        delegationManager.registerAsOperator(delegationApprover, allocationDelay, _metadataURI);

        // Grant admin permissions to the owner
        permissionController.addPendingAdmin(address(this), admin);
    }

    /// @inheritdoc ITEEOperator
    function registerForOperatorSet(address avs, uint32 operatorSetId, bytes calldata data) external onlyAVSFactory {
        uint32[] memory operatorSetIds = new uint32[](1);
        operatorSetIds[0] = operatorSetId;

        IAllocationManagerTypes.RegisterParams memory params =
            IAllocationManagerTypes.RegisterParams({avs: avs, operatorSetIds: operatorSetIds, data: data});

        // Register as an operator for the provided AVS
        allocationManager.registerForOperatorSets(address(this), params);
    }
}
