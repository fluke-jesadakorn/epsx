// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
 * @title MockMainnetUSDT
 * @dev Mock USDT with hardcoded metadata to allow "etching" without storage initialization.
 */
contract MockMainnetUSDT is ERC20 {
    constructor() ERC20("Tether USD", "USDT") {}

    function name() public pure override returns (string memory) { return "Tether USD"; }
    function symbol() public pure override returns (string memory) { return "USDT"; }
    function decimals() public pure override returns (uint8) { return 18; } // BSC USDT has 18 decimals

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

/**
 * @title MockMainnetUSDC
 * @dev Mock USDC with hardcoded metadata.
 */
contract MockMainnetUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {}

    function name() public pure override returns (string memory) { return "USD Coin"; }
    function symbol() public pure override returns (string memory) { return "USDC"; }
    function decimals() public pure override returns (uint8) { return 18; } // BSC USDC has 18 decimals

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
