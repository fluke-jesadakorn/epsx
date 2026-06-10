// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @dev Minimal interface to IEntryPoint for compile-time referencing.
interface IEntryPoint {
    function getDepositInfo(address account) external view returns (uint112, uint112, uint48, uint48, uint48);
    function depositTo(address account) external payable;
    function withdrawTo(address payable withdrawAddress, uint256 amount) external;
    function handleOps(address[] calldata ops, address beneficiary) external;
}
