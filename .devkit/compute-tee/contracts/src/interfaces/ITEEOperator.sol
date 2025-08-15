// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

interface ITEEOperator {
    /// @notice Thrown when the caller is not the AVSFactory
    error NotAVSFactory();

    /**
     * @notice Registers this operator for specified operator sets of an AVS
     * @param avs The AVS address to register with
     * @param operatorSetId The operator set ID within the AVS to register for
     * @param data Additional data for registration (AVS-specific)
     * @dev This function can only be called by the AVSFactory
     */
    function registerForOperatorSet(address avs, uint32 operatorSetId, bytes calldata data) external;
}
