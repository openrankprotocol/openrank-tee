// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {
    IAllocationManager,
    IAllocationManagerTypes
} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IKeyRegistrar, IKeyRegistrarTypes} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IReleaseManager} from "@eigenlayer-contracts/src/contracts/interfaces/IReleaseManager.sol";
import {IKeyRegistrar} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {OperatorSet} from "@eigenlayer-contracts/src/contracts/libraries/OperatorSetLib.sol";
import {IStrategy} from "@eigenlayer-contracts/src/contracts/interfaces/IStrategy.sol";
import {AVSRegistrarAsIdentifier} from "./base/AVSRegistrarAsIdentifier.sol";

contract TEEAVSRegistrar is AVSRegistrarAsIdentifier {
    /// @dev The ReleaseManager contract from EigenLayer
    IReleaseManager public immutable releaseManager;

    /**
     * @param _allocationManager The AllocationManager contract from EigenLayer
     * @param _permissionController The PermissionController contract from EigenLayer
     * @param _keyRegistrar The KeyRegistrar contract from EigenLayer
     * @param _releaseManager The ReleaseManager contract from EigenLayer
     */
    constructor(
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        IKeyRegistrar _keyRegistrar,
        IReleaseManager _releaseManager
    ) AVSRegistrarAsIdentifier(address(0), _allocationManager, _permissionController, _keyRegistrar) {
        releaseManager = _releaseManager;
    }

    /**
     * @notice Initializes the TEE AVS Registrar
     * @dev This function relies on the initializer modifier in super.initialize
     * @param _admin The admin for the AVS
     * @param metadataURI The metadataURI for the AVS
     * @param operatorSetID The operator set ID used by the AVS
     */
    function initialize(address _admin, string memory metadataURI, uint32 operatorSetID) public {
        super.initialize(_admin, metadataURI);

        // Create the operator set for the AVS
        IAllocationManagerTypes.CreateSetParams[] memory params = new IAllocationManagerTypes.CreateSetParams[](1);
        params[0] =
            IAllocationManagerTypes.CreateSetParams({operatorSetId: operatorSetID, strategies: new IStrategy[](0)});
        allocationManager.createOperatorSets(address(this), params);

        // Configure the operator set and register the key
        OperatorSet memory operatorSet = OperatorSet({avs: address(this), id: operatorSetID});
        keyRegistrar.configureOperatorSet(operatorSet, IKeyRegistrarTypes.CurveType.ECDSA);

        // Publish the operator set metadata URI
        releaseManager.publishMetadataURI(operatorSet, metadataURI);
    }
}
