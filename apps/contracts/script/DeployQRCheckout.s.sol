// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console2} from "forge-std/Script.sol";
import {QRCheckout} from "../contracts/QRCheckout.sol";

/// Compile/simulate with Foundry; operators may submit the same creation data
/// through the browser deployment console and MetaMask.
contract DeployQRCheckout is Script {
    function run() external returns (QRCheckout checkout) {
        uint256 chain = vm.envUint("PAY_MERCHANT_EXPECTED_CHAIN_ID");
        require(block.chainid == chain && (chain == 31337 || chain == 97 || chain == 56), "Wrong chain");
        address deployer = vm.envAddress("PAY_MERCHANT_DEPLOYER");
        address treasury = vm.envAddress("PAY_MERCHANT_TREASURY");
        require(deployer != address(0) && treasury != address(0), "Wallet required");
        vm.startBroadcast(deployer);
        checkout = new QRCheckout(treasury);
        vm.stopBroadcast();
        require(checkout.VERSION() == 1 && checkout.FEE_BPS() == 50, "Unexpected contract");
        console2.log("Chain", chain);
        console2.log("QR checkout v1, 50 bps", address(checkout));
        console2.log("Treasury", treasury);
    }
}
