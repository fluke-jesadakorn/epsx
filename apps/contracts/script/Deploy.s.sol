// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/PaymentEscrow.sol";
import "../contracts/MockERC20.sol";

contract DeployScript is Script {
    function run() external {
        vm.startBroadcast();

        console.log("Deploying PaymentEscrow contract...");

        // Get deployer account
        address deployer = msg.sender;
        console.log("Deploying with account:", deployer);

        uint256 deployerBalance = deployer.balance;
        console.log("Account balance:", deployerBalance, "wei");

        // Get admin wallet address from environment
        address adminWallet = vm.envOr("ADMIN_WALLET_ADDRESS", deployer);
        console.log("Admin wallet:", adminWallet);

        // Deploy PaymentEscrow contract
        console.log("Deploying PaymentEscrow...");
        PaymentEscrow paymentEscrow = new PaymentEscrow(deployer);
        address contractAddress = address(paymentEscrow);
        console.log("PaymentEscrow deployed to:", contractAddress);

        // Get chain ID to determine token addresses
        uint256 chainId = block.chainid;
        console.log("Network Chain ID:", chainId);

        address usdtAddress;
        address usdcAddress;

        if (chainId == 56) {
            // BSC Mainnet
            usdtAddress = 0x55d398326f99059fF775485246999027B3197955;
            usdcAddress = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;
            console.log("Network: BSC Mainnet (56)");
        } else if (chainId == 97) {
            // BSC Testnet
            usdtAddress = 0x337610d27c682E347C9cD60BD4b3b107C9d34dDd;
            usdcAddress = 0x64544969ed7EBf5f083679233325356EbE738930;
            console.log("Network: BSC Testnet (97)");
        } else {
            // Local/Development network - deploy mock tokens
            console.log("Network: Local Development (", chainId, ")");
            console.log("Deploying Mock ERC20 tokens for development...");

            MockERC20 usdt = new MockERC20("Tether USD", "USDT", 6);
            usdtAddress = address(usdt);
            console.log("Mock USDT deployed to:", usdtAddress);

            MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
            usdcAddress = address(usdc);
            console.log("Mock USDC deployed to:", usdcAddress);
        }

        console.log("USDT Address:", usdtAddress);
        console.log("USDC Address:", usdcAddress);

        // Configure tokens
        console.log("Enabling tokens...");
        paymentEscrow.setTokenEnabled(usdtAddress, true);
        console.log("USDT enabled");
        paymentEscrow.setTokenEnabled(usdcAddress, true);
        console.log("USDC enabled");

        console.log("Contract configured as payment gateway - plan prices managed by backend");

        // Transfer roles if admin wallet is different from deployer
        if (adminWallet != deployer) {
            console.log("Transferring roles to admin wallet:", adminWallet);
            bytes32 DEFAULT_ADMIN_ROLE = paymentEscrow.DEFAULT_ADMIN_ROLE();
            bytes32 MANAGER_ROLE = paymentEscrow.MANAGER_ROLE();

            paymentEscrow.grantRole(DEFAULT_ADMIN_ROLE, adminWallet);
            paymentEscrow.grantRole(MANAGER_ROLE, adminWallet);
            console.log("Roles granted to admin wallet");
        }

        vm.stopBroadcast();

        // Log deployment summary
        console.log("Deployment complete!");
        console.log("Summary:");
        console.log("Contract Address:", contractAddress);
        console.log("Admin Wallet:", adminWallet);
        console.log("USDT:", usdtAddress);
        console.log("USDC:", usdcAddress);
        console.log("Chain ID:", chainId);
    }
}