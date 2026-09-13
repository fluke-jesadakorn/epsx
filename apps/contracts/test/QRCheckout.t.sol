// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "forge-std/Test.sol";
import "../contracts/QRCheckout.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
contract QRCoin is ERC20 {
    constructor() ERC20("Mock USDT", "USDT") {}
    function mint(address to, uint256 amount) external { _mint(to, amount); }
}
contract QRCheckoutTest is Test {
    QRCheckout factory;
    QRCoin token;
    address merchant=address(0xBEEF);
    address buyer=address(0xA11CE);
    address treasury=address(0xFEE);
    uint256 amount=5 ether;
    bytes32 salt=keccak256("invoice-1");
    function setUp() public { factory=new QRCheckout(treasury); token=new QRCoin(); token.mint(buyer,100 ether); }
    function testOrdinaryTransferNeedsNoApprovalAndAnyoneCanSettle() public {
        address receiver=factory.receiver(merchant,address(token),amount,salt);
        assertEq(receiver.code.length,0);
        vm.prank(buyer); token.transfer(receiver,amount);
        factory.collect(merchant,address(token),amount,salt);
        assertEq(token.balanceOf(merchant),4.975 ether);
        assertEq(token.balanceOf(treasury),0.025 ether);
        assertEq(token.balanceOf(receiver),0);
        factory.collect(merchant,address(token),amount,salt);
        assertEq(token.balanceOf(merchant),4.975 ether);
    }
    function testInvoiceCannotBeReboundOrCollectedByAnotherMerchant() public {
        address receiver=factory.receiver(merchant,address(token),amount,salt);
        vm.prank(buyer);token.transfer(receiver,amount);
        factory.collect(address(this),address(token),amount,salt);
        assertEq(token.balanceOf(receiver),amount);
        assertTrue(factory.receiver(merchant,address(token),amount,salt)!=factory.receiver(merchant,address(token),amount,bytes32(uint256(2))));
        assertTrue(factory.receiver(merchant,address(token),amount,salt)!=factory.receiver(merchant,address(token),amount+1,salt));
    }
    function testFullRefundUsesOnlyMerchantsAllowance() public {
        token.mint(merchant,amount);
        vm.prank(merchant);token.approve(address(factory),amount);
        vm.expectRevert();factory.refundTransfer(address(token),amount,salt,buyer); // caller has no balance/allowance
    }
    function testMerchantRefundAndWrongTokenRecovery() public {
        address receiver=factory.receiver(merchant,address(token),amount,salt);
        token.mint(merchant,amount);
        vm.startPrank(merchant);token.approve(address(factory),amount);factory.refundTransfer(address(token),amount,salt,buyer);vm.stopPrank();
        assertEq(token.balanceOf(buyer),105 ether);
        QRCoin other=new QRCoin();other.mint(receiver,10 ether);
        factory.collect(merchant,address(token),amount,salt);
        QRReceiver(receiver).collect(address(other));
        assertEq(other.balanceOf(merchant),9.95 ether);
    }
    function testNoZeroTreasury() public {vm.expectRevert();new QRCheckout(address(0));}
}
