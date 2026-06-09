// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./PaymentEscrow.sol";
import "./SubscriptionVault.sol";
import "./Paymaster.sol";
import "./TokenRegistry.sol";
import "forge-std/Script.sol";

contract Deploy is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        // BSC mainnet addresses
        address usdt = 0x55d398326f99059fF775485246999027B3197955;
        address usdc = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;
        address entryPoint = 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789;

        vm.startBroadcast(deployerPrivateKey);

        // 1. TokenRegistry
        TokenRegistry registry = new TokenRegistry();
        uint256[] memory bscChains = new uint256[](1);
        bscChains[0] = 56;
        registry.registerToken(usdt, "USDT", 18, 1e15, 1e30, bscChains);
        registry.registerToken(usdc, "USDC", 18, 1e15, 1e30, bscChains);

        // 2. PaymentEscrow
        PaymentEscrow escrow = new PaymentEscrow(deployer, 30); // 0.3% fee

        // 3. SubscriptionVault
        SubscriptionVault vault = new SubscriptionVault();

        // 4. Paymaster
        Paymaster paymaster = new Paymaster(entryPoint, deployer);

        vm.stopBroadcast();

        console.log("TokenRegistry:        ", address(registry));
        console.log("PaymentEscrow:        ", address(escrow));
        console.log("SubscriptionVault:     ", address(vault));
        console.log("Paymaster:            ", address(paymaster));
    }
}
