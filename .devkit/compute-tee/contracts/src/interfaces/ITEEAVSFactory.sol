// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

interface ITEEAVSFactory {
    /// @notice Thrown when an invalid admin address is provided (e.g., zero address)
    error InvalidAdmin();

    /// @notice Thrown when an invalid metadata URI is provided (e.g., empty string)
    error InvalidMetadataURI();

    /// @notice Thrown when an invalid signer is used for a specific AVS
    error InvalidSignerForAVS(address avs);

    /// @notice Emitted when a new AVS is successfully created
    event AVSCreated(address indexed creator, address indexed admin, address indexed avsAddress, string metadataURI);

    /// @notice Emitted when an AVS release is deployed with attestation
    event AVSReleaseDeployed(address indexed avs, uint256 releaseId, bytes signature, bytes attestation);

    /**
     * @notice Creates a new AVS instance
     * @param admin The admin address for the new AVS
     * @param metadataURI The metadata URI for the AVS
     * @return avs The address of the newly created AVS
     */
    function createAVS(address admin, string memory metadataURI) external returns (address avs);

    /**
     * @notice Handles notification of an AVS release deployment with signature verification
     * @dev Validates the signature against the calculated digest hash using the registered signer for the AVS
     * @param avs The AVS address that deployed the release
     * @param releaseId The unique identifier for the release
     * @param signature The signature from the authorized signer for verification
     * @param attestation The attestation data for the release deployment
     */
    function onAVSReleaseDeployed(address avs, uint256 releaseId, bytes calldata signature, bytes calldata attestation)
        external;

    /**
     * @notice Calculates the digest hash for an AVS release deployment
     * @param avs The AVS address
     * @param releaseId The release ID
     * @param attestation The attestation bytes
     * @return The digest hash
     */
    function calculateAVSReleaseDeployedDigestHash(address avs, uint256 releaseId, bytes calldata attestation)
        external
        view
        returns (bytes32);
}
