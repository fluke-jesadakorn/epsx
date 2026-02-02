// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/BEP20Mock.sol";
import "../contracts/PaymentEscrow.sol";

/**
 * @title DeployMocks
 * @dev Deploys Mock USDT and Mock USDC to Testnet and enables them in PaymentEscrow.
 */
contract DeployMocks is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        // Use the PaymentEscrow address from env or deployments
        address escrowAddress = vm.envAddress("PAYMENT_ESCROW_CONTRACT_TESTNET");

        vm.startBroadcast(deployerPrivateKey);

        console.log("=== EPSX Mock Token Deployment ===");
        console.log("Deployer:", deployer);
        console.log("PaymentEscrow:", escrowAddress);

        // 1. Deploy Mock USDT
        console.log("Deploying Mock USDT...");
        MockUSDT usdt = new MockUSDT();
        address usdtAddress = address(usdt);
        console.log("Mock USDT deployed to:", usdtAddress);

        // 2. Deploy Mock USDC
        console.log("Deploying Mock USDC...");
        MockUSDC usdc = new MockUSDC();
        address usdcAddress = address(usdc);
        console.log("Mock USDC deployed to:", usdcAddress);

        // 3. Enable in PaymentEscrow
        if (escrowAddress != address(0)) {
            console.log("Enabling mocks in PaymentEscrow...");
            PaymentEscrow escrow = PaymentEscrow(payable(escrowAddress));
            escrow.setTokenEnabled(usdtAddress, true);
            escrow.setTokenEnabled(usdcAddress, true);
            console.log("Mocks enabled successfully!");
        }

        vm.stopBroadcast();

        console.log("");
        console.log("=== Deployment Complete ===");
        console.log("Mock USDT:", usdtAddress);
        console.log("Mock USDC:", usdcAddress);
        console.log("");
        console.log("NOTE: You can now mint tokens using:");
        console.log("cast send", usdtAddress);
        console.log("\"mint(address,uint256)\"");
        console.log(deployer, "1000000000000000000000 --rpc-url $RPC --private-key $PK");
    }
}
