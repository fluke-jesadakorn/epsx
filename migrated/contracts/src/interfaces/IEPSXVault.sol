// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IEPSXVault {
    function deposit(address token, uint256 amount) external;
    function withdraw(address token, uint256 amount) external;
    function chargeSubscription(bytes32 subId, uint256 amount) external;
    function claimEarnings(address token) external;
    function balance(address user, address token) external view returns (uint256);
    function earnings(address merchant, address token) external view returns (uint256);
}
