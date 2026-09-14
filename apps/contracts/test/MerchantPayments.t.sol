// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "forge-std/Test.sol";
import "../contracts/MerchantPayments.sol";
import {TestCoin, RejectReceiver} from "./DealEscrow.t.sol";

contract MerchantReentry {
    DirectPayments public router; bytes32 public id; bool public tried; bool public succeeded;
    function setup(DirectPayments r, bytes32 i) external {router=r;id=i;}
    receive() external payable {tried=true;(succeeded,)=address(router).call(abi.encodeCall(router.refund,(id)));}
}
contract MerchantPaymentsTest is Test {
    DirectPayments direct; MerchantEscrow escrow; TestCoin coin;
    address payer=address(0x10011);address merchant=address(0x10022);address admin=address(0x10033);address treasury=address(0x10044);address stranger=address(0x10055);
    uint256 deadline;
    function setUp() public {
        coin=new TestCoin();address[] memory tokens=new address[](1);tokens[0]=address(coin);
        direct=new DirectPayments(admin,treasury,tokens);escrow=new MerchantEscrow(admin,treasury,tokens);
        deadline=block.timestamp+1800;vm.deal(payer,100 ether);vm.deal(merchant,100 ether);coin.mint(payer,100 ether);coin.mint(merchant,100 ether);
        vm.startPrank(payer);coin.approve(address(direct),type(uint256).max);coin.approve(address(escrow),type(uint256).max);vm.stopPrank();
        vm.prank(merchant);coin.approve(address(direct),type(uint256).max);
    }
    function pay(address token,uint256 amount) internal returns(bytes32){vm.prank(payer);return direct.pay{value:token==address(0)?amount:0}(merchant,token,amount,deadline,0);}
    function fund(address token,uint256 amount,bytes32 salt) internal returns(bytes32){vm.prank(payer);return escrow.deposit{value:token==address(0)?amount:0}(merchant,token,amount,deadline,salt);}
    function testDirectNativeAtomicSplitAndFullRefund() public {
        bytes32 id=pay(address(0),1 ether);assertEq(merchant.balance,100 ether+995e15);assertEq(treasury.balance,5e15);assertEq(address(direct).balance,0);
        vm.prank(merchant);direct.refund{value:1 ether}(id);assertEq(payer.balance,100 ether);assertEq(treasury.balance,5e15);assertEq(merchant.balance,100 ether-5e15);
        vm.prank(merchant);vm.expectRevert(MerchantPayments.InvalidState.selector);direct.refund{value:1 ether}(id);
    }
    function testDirectTokenRefundUsesMerchantFundsAndRetainsFee() public {
        bytes32 id=pay(address(coin),10000);assertEq(coin.balanceOf(treasury),50);
        vm.prank(merchant);direct.refund(id);assertEq(coin.balanceOf(payer),100 ether);assertEq(coin.balanceOf(merchant),100 ether-50);assertEq(coin.balanceOf(address(direct)),0);
    }
    function testNoDuplicatePaymentOrForeignRefund() public {
        bytes32 id=pay(address(0),1 ether);vm.prank(payer);vm.expectRevert(MerchantPayments.InvalidState.selector);direct.pay{value:1 ether}(merchant,address(0),1 ether,deadline,0);
        vm.prank(stranger);vm.expectRevert(MerchantPayments.Unauthorized.selector);direct.refund(id);
        vm.prank(payer);vm.expectRevert(MerchantPayments.InvalidState.selector);direct.release(id);
    }
    function testEveryTermAndPayerBoundToIdentifier() public view {
        bytes32 id=direct.paymentId(payer,merchant,address(0),100,deadline,0);
        assertTrue(id!=direct.paymentId(stranger,merchant,address(0),100,deadline,0));assertTrue(id!=direct.paymentId(payer,stranger,address(0),100,deadline,0));
        assertTrue(id!=direct.paymentId(payer,merchant,address(coin),100,deadline,0));assertTrue(id!=direct.paymentId(payer,merchant,address(0),101,deadline,0));
        assertTrue(id!=direct.paymentId(payer,merchant,address(0),100,deadline+1,0));assertTrue(id!=direct.paymentId(payer,merchant,address(0),100,deadline,bytes32(uint256(1))));
        assertTrue(id!=escrow.paymentId(payer,merchant,address(0),100,deadline,0));
    }
    function testExpiryIsEnforcedOnChainForBothModes() public {
        vm.warp(deadline+1);vm.prank(payer);vm.expectRevert(MerchantPayments.InvalidTerms.selector);direct.pay{value:1}(merchant,address(0),1,deadline,0);
        vm.prank(payer);vm.expectRevert(MerchantPayments.InvalidTerms.selector);escrow.deposit{value:1}(merchant,address(0),1,deadline,0);
    }
    function testEscrowFeeOnlyOnReleaseAndLiabilities() public {
        bytes32 id=fund(address(coin),10000,0);fund(address(coin),20000,bytes32(uint256(1)));assertEq(coin.balanceOf(treasury),0);assertEq(escrow.liabilities(address(coin)),30000);
        vm.prank(payer);escrow.release(id);assertEq(coin.balanceOf(treasury),100);assertEq(escrow.liabilities(address(coin)),20000);assertEq(coin.balanceOf(address(escrow)),20000);
    }
    function testEscrowDisputeAndResolutionPermissions() public {
        bytes32 id=fund(address(0),1 ether,0);vm.prank(admin);vm.expectRevert(MerchantPayments.InvalidState.selector);escrow.resolve(id,true);
        vm.prank(stranger);vm.expectRevert(MerchantPayments.Unauthorized.selector);escrow.dispute(id);
        vm.prank(payer);escrow.dispute(id);vm.prank(payer);vm.expectRevert(MerchantPayments.InvalidState.selector);escrow.release(id);
        vm.prank(merchant);vm.expectRevert(MerchantPayments.Unauthorized.selector);escrow.resolve(id,true);
        vm.prank(admin);escrow.resolve(id,true);assertEq(treasury.balance,1e16);assertEq(escrow.liabilities(address(0)),0);
    }
    function testPayeeAndAdminEscrowRefundFullWithNoFee() public {
        bytes32 a=fund(address(0),1 ether,0);bytes32 b=fund(address(0),2 ether,bytes32(uint256(1)));
        vm.prank(payer);escrow.dispute(a);vm.prank(merchant);escrow.refund(a);
        vm.prank(merchant);escrow.dispute(b);vm.prank(admin);escrow.resolve(b,false);assertEq(payer.balance,100 ether);assertEq(treasury.balance,0);
    }
    function testPauseLeavesRefundsAndExistingEscrowAvailable() public {
        bytes32 a=pay(address(0),1 ether);bytes32 b=fund(address(0),1 ether,0);
        vm.prank(admin);direct.setPaused(true);vm.prank(admin);escrow.setPaused(true);
        vm.prank(payer);vm.expectRevert();direct.pay{value:2}(merchant,address(0),2,deadline,bytes32(uint256(1)));
        vm.prank(payer);vm.expectRevert();escrow.deposit{value:2}(merchant,address(0),2,deadline,bytes32(uint256(1)));
        vm.prank(merchant);direct.refund{value:1 ether}(a);vm.prank(payer);escrow.release(b);
    }
    function testBadTokenAmountAndFailedTransfersAreAtomic() public {
        TestCoin other=new TestCoin();vm.prank(payer);vm.expectRevert(MerchantPayments.UnsupportedToken.selector);direct.pay(merchant,address(other),1,deadline,0);
        vm.prank(payer);vm.expectRevert(MerchantPayments.IncorrectAmount.selector);direct.pay{value:2}(merchant,address(0),1,deadline,0);
        coin.failures(false,true);vm.prank(payer);vm.expectRevert(MerchantPayments.IncorrectAmount.selector);direct.pay(merchant,address(coin),10000,deadline,0);
        coin.failures(false,false);bytes32 id=fund(address(coin),10000,0);coin.failures(true,false);vm.prank(payer);vm.expectRevert();escrow.release(id);assertEq(escrow.liabilities(address(coin)),10000);
        RejectReceiver receiver=new RejectReceiver();vm.prank(payer);vm.expectRevert(MerchantPayments.TransferFailed.selector);direct.pay{value:1 ether}(address(receiver),address(0),1 ether,deadline,0);
    }
    function testReentrancyCannotRefundDuringDirectSettlement() public {
        MerchantReentry receiver=new MerchantReentry();bytes32 id=direct.paymentId(payer,address(receiver),address(0),1 ether,deadline,0);receiver.setup(direct,id);
        vm.prank(payer);direct.pay{value:1 ether}(address(receiver),address(0),1 ether,deadline,0);assertTrue(receiver.tried());assertFalse(receiver.succeeded());
    }
    function testMaximumAmountDoesNotOverflow() public {
        TestCoin huge=new TestCoin();address[] memory tokens=new address[](1);tokens[0]=address(huge);DirectPayments router=new DirectPayments(admin,treasury,tokens);
        huge.mint(payer,type(uint256).max);vm.startPrank(payer);huge.approve(address(router),type(uint256).max);router.pay(merchant,address(huge),type(uint256).max,deadline,0);vm.stopPrank();
        assertEq(huge.balanceOf(merchant)+huge.balanceOf(treasury),type(uint256).max);
    }
    function testFuzzFees(uint256 amount) public view {assertEq(direct.feeFor(amount),amount/10000*50+(amount%10000)*50/10000);assertEq(escrow.feeFor(amount),amount/10000*100+(amount%10000)*100/10000);}
    function testFuzzConservation(uint96 raw,bool escrowMode,bool refund_) public {
        uint256 amount=bound(uint256(raw),1,100 ether);bytes32 id;
        if(escrowMode){id=fund(address(coin),amount,0);vm.prank(refund_?merchant:payer);if(refund_)escrow.refund(id);else escrow.release(id);}
        else{id=pay(address(coin),amount);if(refund_){vm.prank(merchant);direct.refund(id);}}
        assertEq(coin.balanceOf(payer)+coin.balanceOf(merchant)+coin.balanceOf(treasury)+coin.balanceOf(address(escrow))+coin.balanceOf(address(direct)),200 ether);
    }
}
