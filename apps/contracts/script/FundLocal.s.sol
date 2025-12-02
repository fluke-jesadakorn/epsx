// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract FundLocal is Script, Test {
    function run() external {
        // This script is intended to be run against a local fork of BSC
        // It uses cheatcodes to fund the default account with USDT and USDC

        // Default Anvil Account #0 or from Environment
        address user = vm.envOr("TARGET_ADDRESS", address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266));
        // Or use the broadcaster
        // address user = msg.sender; 

        // BSC Token Addresses
        address usdt = 0x55d398326f99059fF775485246999027B3197955;
        address usdc = 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d;

        console.log("Funding account:", user);

        // Amount: 10,000 tokens
        // BSC-Pegged USDT/USDC have 18 decimals
        uint256 amount = 10000 * 1e18;

        // Deal USDT
        console.log("Setting USDT balance...");
        // stdCheats 'deal' might not be automatically available in Script context depending on version,
        // but generally is. If not, we can use low-level storage manipulation, 
        // but 'deal' is robust for standard ERC20s.
        deal(usdt, user, amount);
        console.log("New USDT Balance:", IERC20(usdt).balanceOf(user));

        // Deal USDC
        console.log("Setting USDC balance...");
        deal(usdc, user, amount);
        console.log("New USDC Balance:", IERC20(usdc).balanceOf(user));

        // Deal BNB (Native)
        console.log("Setting BNB balance...");
        deal(user, 100 ether);
        console.log("New BNB Balance:", user.balance);
    }
}
