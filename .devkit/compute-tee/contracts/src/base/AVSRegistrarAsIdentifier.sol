// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.27;

import {Initializable} from "@openzeppelin-upgrades/contracts/proxy/utils/Initializable.sol";
import {IAVSRegistrar} from "@eigenlayer-contracts/src/contracts/interfaces/IAVSRegistrar.sol";
import {IAVSRegistrarInternal} from "@eigenlayer-middleware/src/interfaces/IAVSRegistrarInternal.sol";
import {IAllocationManager} from "@eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IPermissionController} from "@eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {IKeyRegistrar} from "@eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {AVSRegistrar} from "./AVSRegistrar.sol";

/// @notice An AVSRegistrar that is the identifier for the AVS in EigenLayer core.
/// @dev Once deployed, the `admin` will control other parameters of the AVS, such as creating operatorSets, slashing, etc.
///      This contract varies from the EigenLayer AVSRegistrar in that it does not validate operator keys
///      at the time of registration. Keys are registered lazily by the operator.
contract AVSRegistrarAsIdentifier is AVSRegistrar {
    /// @notice The permission controller for the AVS
    IPermissionController public immutable permissionController;

    /// @dev The immutable avs address `AVSRegistrar` is NOT the address of the AVS in EigenLayer core.
    /// @dev The address of the AVS in EigenLayer core is the proxy contract, and it is set via the `initialize` function below.
    constructor(
        address _avs,
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        IKeyRegistrar _keyRegistrar
    ) AVSRegistrar(_avs, _allocationManager, _keyRegistrar) {
        // Set the permission controller for future interactions
        permissionController = _permissionController;
    }

    /**
     * @notice Initializes the AVSRegistrarAsIdentifier
     * @param admin The admin for the AVS
     * @param metadataURI The metadataURI for the AVS
     * @dev This function enables the address of the AVS in the core protocol to be the proxy AVSRegistrarAsIdentifier contract
     */
    function initialize(address admin, string memory metadataURI) public initializer {
        // Set the metadataURI and the registrar for the AVS to this registrar contract
        allocationManager.updateAVSMetadataURI(address(this), metadataURI);
        allocationManager.setAVSRegistrar(address(this), this);

        // Set the admin for the AVS
        permissionController.addPendingAdmin(address(this), admin);
    }

    /// @inheritdoc IAVSRegistrar
    function supportsAVS(address _avs) public view override returns (bool) {
        return _avs == address(this);
    }

    /// @inheritdoc IAVSRegistrarInternal
    function getAVS() external view override returns (address) {
        return address(this);
    }
}
