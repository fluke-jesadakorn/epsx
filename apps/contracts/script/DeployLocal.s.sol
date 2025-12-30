// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/PaymentEscrow.sol";

/**
 * @title DeployLocal
 * @dev Deploys PaymentEscrow and registers Mainnet token addresses.
 * 
 * Note: This script DOES NOT deploy the tokens itself.
 * The tokens (USDT/USDC) at Mainnet addresses must be "etched" onto the local chain
 * using the separate `etch-tokens.sh` script or `bun anvil:etch`.
 */
contract DeployLocal is Script {
    // Anvil default accounts
    address constant ACCOUNT_0 = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    
    // Official BSC Mainnet Addresses
    address constant MAINNET_USDT = 0x55d398326f99059fF775485246999027B3197955;
    address constant MAINNET_USDC = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;

    function run() external {
        uint256 deployerPrivateKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        
        vm.startBroadcast(deployerPrivateKey);

        console.log("=== Deploying Payment Escrow (Local) ===");

        // 1. Deploy PaymentEscrow
        PaymentEscrow escrow = new PaymentEscrow(ACCOUNT_0);
        console.log("PaymentEscrow deployed to:", address(escrow));

        // 2. Enable Mainnet Tokens (which will be etched shortly)
        console.log("Enabling Mainnet Token Addresses...");
        escrow.setTokenEnabled(MAINNET_USDT, true);
        escrow.setTokenEnabled(MAINNET_USDC, true);

        vm.stopBroadcast();

        console.log("");
        console.log("=== Escrow Setup Complete ===");
        console.log("Now execute 'bun anvil:etch' to plant token code.");
    }
}


