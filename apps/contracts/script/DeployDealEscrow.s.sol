// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contracts/DealEscrow.sol";

/// @notice Separate Pay deployment; never touches the legacy package contract.
/// @dev Sign with a separately configured Foundry wallet or hardware wallet.
/// No private key is read from application configuration or embedded here.
contract DeployDealEscrow is Script {
    function run() external returns (DealEscrow escrow) {
        uint256 expectedChain = vm.envUint("PAY_ESCROW_EXPECTED_CHAIN_ID");
        require(block.chainid == expectedChain, "Wrong deployment chain");
        require(expectedChain == 56 || expectedChain == 97 || expectedChain == 31337, "Unsupported chain");
        address deployer = vm.envAddress("PAY_ESCROW_DEPLOYER");
        address admin = vm.envAddress("PAY_ESCROW_ADMIN");
        address treasury = vm.envAddress("PAY_ESCROW_TREASURY");
        string memory rawTokens = vm.envString("PAY_ESCROW_TOKEN_ADDRESSES");
        address[] memory tokens = bytes(rawTokens).length == 0
            ? new address[](0)
            : vm.envAddress("PAY_ESCROW_TOKEN_ADDRESSES", ",");
        require(deployer != address(0), "Deployer required");
        vm.startBroadcast(deployer);
        escrow = new DealEscrow(admin, treasury, tokens);
        vm.stopBroadcast();
        console2.log("Pay escrow chain", block.chainid);
        console2.log("Pay escrow contract", address(escrow));
        console2.log("Pay escrow version", escrow.VERSION());
        console2.log("Arbiter", escrow.arbiter());
        console2.log("Treasury", escrow.treasury());
    }
}
