// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import {Script, console2} from "forge-std/Script.sol";
import {DirectPayments, MerchantEscrow} from "../contracts/MerchantPayments.sol";

/// Separate contracts. No existing escrow or package deployment is changed.
/// Foundry obtains signing authority from the operator's selected wallet, never the backend.
contract DeployMerchantPayments is Script {
    function run() external returns (DirectPayments direct, MerchantEscrow escrow) {
        uint256 chain = vm.envUint("PAY_MERCHANT_EXPECTED_CHAIN_ID");
        require(block.chainid == chain && (chain == 31337 || chain == 97 || chain == 56), "Wrong chain");
        address deployer = vm.envAddress("PAY_MERCHANT_DEPLOYER");
        address admin = vm.envAddress("PAY_MERCHANT_ADMIN");
        address treasury = vm.envAddress("PAY_MERCHANT_TREASURY");
        string memory raw = vm.envString("PAY_MERCHANT_TOKEN_ADDRESSES");
        address[] memory tokens = bytes(raw).length == 0 ? new address[](0) : vm.envAddress("PAY_MERCHANT_TOKEN_ADDRESSES", ",");
        require(deployer != address(0), "Deployer required");
        vm.startBroadcast(deployer);
        direct = new DirectPayments(admin, treasury, tokens);
        escrow = new MerchantEscrow(admin, treasury, tokens);
        vm.stopBroadcast();
        console2.log("Chain", chain);
        console2.log("Direct v1, 50 bps", address(direct));
        console2.log("Merchant escrow v2, 100 bps", address(escrow));
        console2.log("Admin", admin);
        console2.log("Treasury", treasury);
    }
}
