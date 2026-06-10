// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/PaymentEscrow.sol";
import "../src/SubscriptionVault.sol";
import "../src/TokenRegistry.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockUSDT is ERC20 {
    constructor() ERC20("Mock USDT", "USDT") {}
    function mint(address to, uint256 amount) external { _mint(to, amount); }
}

contract PaymentEscrowTest is Test {
    PaymentEscrow escrow;
    MockUSDT usdt;
    address buyer = address(1);
    address seller = address(2);
    address feeRecipient = address(3);

    function setUp() public {
        usdt = new MockUSDT();
        escrow = new PaymentEscrow(feeRecipient, 30);
        usdt.mint(buyer, 1000e18);
    }

    function testCreateAndRelease() public {
        vm.startPrank(buyer);
        usdt.approve(address(escrow), 100e18);
        bytes32 orderId = keccak256("order-1");
        bytes32 escrowId = escrow.createEscrow(seller, address(usdt), 100e18, orderId);
        vm.stopPrank();

        assertEq(usdt.balanceOf(address(escrow)), 100e18);

        vm.prank(buyer);
        escrow.releaseEscrow(escrowId);

        // 30 bps fee = 0.3% of 100 = 0.3
        uint256 fee = (100e18 * 30) / 10000;
        assertEq(usdt.balanceOf(seller), 100e18 - fee);
        assertEq(usdt.balanceOf(feeRecipient), fee);
    }

    function testRefund() public {
        vm.startPrank(buyer);
        usdt.approve(address(escrow), 100e18);
        bytes32 escrowId = escrow.createEscrow(seller, address(usdt), 100e18, keccak256("o2"));
        escrow.refundEscrow(escrowId);
        vm.stopPrank();

        assertEq(usdt.balanceOf(buyer), 1000e18);
    }

    function testCannotDoubleRelease() public {
        vm.startPrank(buyer);
        usdt.approve(address(escrow), 100e18);
        bytes32 escrowId = escrow.createEscrow(seller, address(usdt), 100e18, keccak256("o3"));
        escrow.releaseEscrow(escrowId);
        vm.expectRevert(PaymentEscrow.EscrowNotActive.selector);
        escrow.releaseEscrow(escrowId);
        vm.stopPrank();
    }
}

contract SubscriptionVaultTest is Test {
    SubscriptionVault vault;
    MockUSDT usdt;
    address merchant = address(10);
    address subscriber = address(11);

    function setUp() public {
        usdt = new MockUSDT();
        vault = new SubscriptionVault();
        usdt.mint(subscriber, 10000e18);
    }

    function testSubscribeAndCharge() public {
        vm.prank(merchant);
        uint64 planId = vault.createPlan(address(usdt), 100e18, 30 days, 2);

        vm.prank(subscriber);
        uint64 subId = vault.subscribe(planId);

        vm.startPrank(subscriber);
        usdt.approve(address(vault), 100e18);
        vault.charge(subId, 1);
        vm.stopPrank();

        assertEq(vault.merchantBalances(merchant), 100e18);
    }

    function testWithdraw() public {
        vm.prank(merchant);
        uint64 planId = vault.createPlan(address(usdt), 100e18, 30 days, 2);

        vm.prank(subscriber);
        uint64 subId = vault.subscribe(planId);

        vm.startPrank(subscriber);
        usdt.approve(address(vault), 100e18);
        vault.charge(subId, 1);
        vm.stopPrank();

        vm.prank(merchant);
        vault.withdraw(address(usdt));

        assertEq(usdt.balanceOf(merchant), 100e18);
    }

    function testCancel() public {
        vm.prank(merchant);
        uint64 planId = vault.createPlan(address(usdt), 100e18, 30 days, 2);

        vm.prank(subscriber);
        uint64 subId = vault.subscribe(planId);

        vm.prank(subscriber);
        vault.cancel(subId);

        SubscriptionVault.Subscription memory s = vault.getSubscription(subId);
        assertEq(uint(s.status), 1); // Cancelled
    }
}

contract TokenRegistryTest is Test {
    TokenRegistry registry;
    MockUSDT usdt;

    function setUp() public {
        registry = new TokenRegistry();
        usdt = new MockUSDT();
    }

    function testRegisterAndCheck() public {
        uint256[] memory chains = new uint256[](1);
        chains[0] = 56;
        registry.registerToken(address(usdt), "USDT", 18, 1, 1e30, chains);

        assertTrue(registry.isAccepted(56, address(usdt)));
        assertFalse(registry.isAccepted(1, address(usdt)));
    }
}
