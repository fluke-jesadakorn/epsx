// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
 * @title BEP20Mock
 * @dev BEP20-compatible mock token for local Anvil development.
 * 
 * This contract is used to "etch" bytecode onto mainnet token addresses
 * (USDT/USDC) in local Anvil chains, allowing consistent addresses across
 * all environments (local, testnet, mainnet).
 * 
 * Usage:
 *   1. Deploy this contract to get its bytecode
 *   2. Use `anvil_setCode` to etch bytecode to mainnet addresses
 *   3. Call `mint()` to fund test accounts
 */
contract BEP20Mock is ERC20 {
    uint8 private immutable _decimals;
    string private _tokenName;
    string private _tokenSymbol;

    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_
    ) ERC20(name_, symbol_) {
        _tokenName = name_;
        _tokenSymbol = symbol_;
        _decimals = decimals_;
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    /**
     * @dev Mint tokens to any address. Only available on local networks.
     * @param to The address to mint tokens to
     * @param amount The amount of tokens to mint (in wei, with decimals)
     */
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    /**
     * @dev Burn tokens from any address. Only available on local networks.
     * @param from The address to burn tokens from
     * @param amount The amount of tokens to burn
     */
    function burn(address from, uint256 amount) external {
        _burn(from, amount);
    }
}

/**
 * @title MockUSDT
 * @dev Hardcoded USDT mock for etching without storage initialization.
 * BSC-Pegged USDT uses 18 decimals.
 */
contract MockUSDT is ERC20 {
    constructor() ERC20("Tether USD", "USDT") {}

    function name() public pure override returns (string memory) { return "Tether USD"; }
    function symbol() public pure override returns (string memory) { return "USDT"; }
    function decimals() public pure override returns (uint8) { return 18; }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

/**
 * @title MockUSDC
 * @dev Hardcoded USDC mock for etching without storage initialization.
 * BSC-Pegged USDC uses 18 decimals.
 */
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {}

    function name() public pure override returns (string memory) { return "USD Coin"; }
    function symbol() public pure override returns (string memory) { return "USDC"; }
    function decimals() public pure override returns (uint8) { return 18; }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
