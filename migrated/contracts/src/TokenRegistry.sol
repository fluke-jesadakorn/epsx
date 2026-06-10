// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title TokenRegistry
 * @notice Per-chain registry of accepted payment tokens.
 */
contract TokenRegistry is Ownable {
    struct TokenInfo {
        address token;
        string symbol;
        uint8 decimals;
        bool active;
        uint256 minAmount;
        uint256 maxAmount;
    }

    mapping(address => TokenInfo) public tokens;
    mapping(uint256 => mapping(address => bool)) public chainToken; // chainId => token => active
    address[] public tokenList;

    event TokenRegistered(address indexed token, string symbol, uint8 decimals, uint256 minAmount, uint256 maxAmount);
    event TokenUpdated(address indexed token, bool active);
    event TokenRemoved(address indexed token);

    error ZeroAddress();
    error TokenNotFound();
    error AlreadyRegistered();

    constructor() Ownable(msg.sender) {}

    function registerToken(
        address token,
        string calldata symbol,
        uint8 decimals,
        uint256 minAmount,
        uint256 maxAmount,
        uint256[] calldata chainIds
    ) external onlyOwner {
        if (token == address(0)) revert ZeroAddress();
        if (tokens[token].token != address(0)) revert AlreadyRegistered();

        tokens[token] = TokenInfo({
            token: token,
            symbol: symbol,
            decimals: decimals,
            active: true,
            minAmount: minAmount,
            maxAmount: maxAmount
        });
        tokenList.push(token);

        for (uint256 i = 0; i < chainIds.length; i++) {
            chainToken[chainIds[i]][token] = true;
        }

        emit TokenRegistered(token, symbol, decimals, minAmount, maxAmount);
    }

    function setActive(address token, bool active) external onlyOwner {
        if (tokens[token].token == address(0)) revert TokenNotFound();
        tokens[token].active = active;
        emit TokenUpdated(token, active);
    }

    function removeToken(address token) external onlyOwner {
        if (tokens[token].token == address(0)) revert TokenNotFound();
        delete tokens[token];
        emit TokenRemoved(token);
    }

    function isAccepted(uint256 chainId, address token) external view returns (bool) {
        return chainToken[chainId][token] && tokens[token].active;
    }

    function getTokenInfo(address token) external view returns (TokenInfo memory) {
        return tokens[token];
    }

    function listTokens() external view returns (address[] memory) {
        return tokenList;
    }
}
