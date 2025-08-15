// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {IAllocationManager} from "eigenlayer-contracts/src/contracts/interfaces/IAllocationManager.sol";
import {IPermissionController} from "eigenlayer-contracts/src/contracts/interfaces/IPermissionController.sol";
import {SignatureUtilsMixin} from "@eigenlayer-contracts/src/contracts/mixins/SignatureUtilsMixin.sol";
import {IKeyRegistrar} from "eigenlayer-contracts/src/contracts/interfaces/IKeyRegistrar.sol";
import {OperatorSet} from "@eigenlayer-contracts/src/contracts/libraries/OperatorSetLib.sol";
import {TEEAVSFactoryStorage} from "./base/TEEAVSFactoryStorage.sol";
import {ITEEAVSFactory} from "./interfaces/ITEEAVSFactory.sol";
import {ITEEOperator} from "./interfaces/ITEEOperator.sol";
import {TEEAVSRegistrar} from "./TEEAVSRegistrar.sol";

contract TEEAVSFactory is TEEAVSFactoryStorage, SignatureUtilsMixin {
    using Clones for address;

    /**
     * @param _version The version string to use for this contract's domain separator
     * @param _avsImplementation The address of the AVSRegistrarAsIdentifier implementation
     * @param _allocationManager The AllocationManager contract address
     * @param _permissionController The PermissionController contract address
     * @param _keyRegistrar The KeyRegistrar contract address
     * @param _teeOperator The TEEOperator contract address
     */
    constructor(
        string memory _version,
        address _avsImplementation,
        IAllocationManager _allocationManager,
        IPermissionController _permissionController,
        IKeyRegistrar _keyRegistrar,
        ITEEOperator _teeOperator
    )
        SignatureUtilsMixin(_version)
        TEEAVSFactoryStorage(_avsImplementation, _allocationManager, _permissionController, _keyRegistrar, _teeOperator)
    {}

    /**
     * @notice Create a new AVS clone
     * @param admin The admin address for the new AVS
     * @param metadataURI The metadata URI for the AVS
     * @return avs The address of the newly created AVS
     */
    function createAVS(address admin, string memory metadataURI) external returns (address avs) {
        require(admin != address(0), InvalidAdmin());
        require(bytes(metadataURI).length != 0, InvalidMetadataURI());

        // Create AVS registrar
        avs = avsImplementation.clone();

        // Initialize the AVS
        TEEAVSRegistrar(avs).initialize(admin, metadataURI, OPERATOR_SET_ID);

        // Register the TEE operator for the AVS operator set
        teeOperator.registerForOperatorSet(avs, OPERATOR_SET_ID, new bytes(0));

        emit AVSCreated(msg.sender, admin, avs, metadataURI);
    }

    /// @inheritdoc ITEEAVSFactory
    function onAVSReleaseDeployed(address avs, uint256 releaseId, bytes calldata signature, bytes calldata attestation)
        external
    {
        // TODO(post-mvp): Verify operator signature or otherwise validate attestation.
        // address signer = keyRegistrar.getECDSAAddress(OperatorSet({avs: avs, id: OPERATOR_SET_ID}), address(teeOperator));
        // require(signer != address(0), InvalidSignerForAVS(avs));
        // _checkIsValidSignatureNow(
        //     signer, calculateAVSReleaseDeployedDigestHash(avs, releaseId, attestation), signature, block.timestamp
        // );

        emit AVSReleaseDeployed(avs, releaseId, signature, attestation);
    }

    /// @inheritdoc ITEEAVSFactory
    function calculateAVSReleaseDeployedDigestHash(address avs, uint256 releaseId, bytes calldata attestation)
        public
        view
        returns (bytes32)
    {
        return _calculateSignableDigest(
            keccak256(abi.encode(AVS_RELEASE_DEPLOYED_TYPEHASH, avs, releaseId, keccak256(attestation)))
        );
    }
}
