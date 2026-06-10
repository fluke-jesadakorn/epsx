// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/PaymentEscrow.sol";

/**
 * @title Deploy
 * @dev Unified deployment script for PaymentEscrow on all networks.
 * 
 * Usage:
 *   # Local Anvil (requires setup-local.sh for token etching)
 *   forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast --private-key 0xac0974bec...
 * 
 *   # BSC Testnet
 *   forge script script/Deploy.s.sol --rpc-url $BSC_TESTNET_RPC --broadcast --private-key $PRIVATE_KEY
 * 
 *   # BSC Mainnet
 *   forge script script/Deploy.s.sol --rpc-url $BSC_MAINNET_RPC --broadcast --private-key $PRIVATE_KEY
 */
contract Deploy is Script {
    // Official BSC Mainnet Token Addresses
    // These addresses are used on all networks for consistency
    address constant USDT = 0x55d398326f99059fF775485246999027B3197955;
    address constant USDC = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;

    function run() external {
        // Get deployer from environment or use Anvil default
        uint256 deployerPrivateKey = vm.envOr(
            "PRIVATE_KEY",
            uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        );
        
        address deployer = vm.addr(deployerPrivateKey);
        address adminWallet = vm.envOr("ADMIN_WALLET_ADDRESS", deployer);

        vm.startBroadcast(deployerPrivateKey);

        console.log("=== EPSX PaymentEscrow Deployment ===");
        console.log("Network Chain ID:", block.chainid);
        console.log("Deployer:", deployer);
        console.log("Admin Wallet:", adminWallet);
        console.log("");

        // Deploy PaymentEscrow
        console.log("Deploying PaymentEscrow...");
        PaymentEscrow escrow = new PaymentEscrow(deployer);
        address escrowAddress = address(escrow);
        console.log("PaymentEscrow deployed to:", escrowAddress);
        console.log("");

        // Enable tokens (same addresses for all networks)
        console.log("Enabling tokens...");
        escrow.setTokenEnabled(USDT, true);
        console.log("USDT enabled:", USDT);
        escrow.setTokenEnabled(USDC, true);
        console.log("USDC enabled:", USDC);
        console.log("");

        // Transfer roles if admin wallet is different from deployer
        if (adminWallet != deployer) {
            console.log("Transferring roles to admin wallet...");
            bytes32 DEFAULT_ADMIN_ROLE = escrow.DEFAULT_ADMIN_ROLE();
            bytes32 MANAGER_ROLE = escrow.MANAGER_ROLE();

            escrow.grantRole(DEFAULT_ADMIN_ROLE, adminWallet);
            escrow.grantRole(MANAGER_ROLE, adminWallet);
            console.log("Roles granted to:", adminWallet);
        }

        vm.stopBroadcast();

        // Summary
        console.log("");
        console.log("=== Deployment Complete ===");
        console.log("PaymentEscrow:", escrowAddress);
        console.log("USDT:", USDT);
        console.log("USDC:", USDC);
        console.log("");

        if (block.chainid == 31337) {
            console.log("NOTE: For local Anvil, run 'bun setup:local' to etch tokens.");
        }
    }
}