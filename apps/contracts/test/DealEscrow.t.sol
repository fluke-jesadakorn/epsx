// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "forge-std/Test.sol";
import "../contracts/DealEscrow.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
contract TestCoin is ERC20 {
    bool public fail;
    bool public taxed;
    constructor() ERC20("Test", "TEST") {}
    function mint(address user, uint256 amount) external { _mint(user, amount); }
    function failures(bool f, bool t) external { fail=f; taxed=t; }
    function transfer(address to, uint256 amount) public override returns (bool) { if (fail) return false; return super.transfer(to, amount); }
    function _update(address from, address to, uint256 amount) internal override {
        if (taxed && from != address(0) && to != address(0)) { super._update(from,address(0),amount/100); super._update(from,to,amount-amount/100); }
        else super._update(from,to,amount);
    }
}
contract RejectReceiver { receive() external payable { revert(); } }
contract ReenterReceiver {
    DealEscrow public escrow; bytes32 public id; bool public attempted; bool public succeeded;
    function configure(DealEscrow e, bytes32 i) external {escrow=e;id=i;}
    receive() external payable { attempted=true; (succeeded,) = address(escrow).call(abi.encodeCall(escrow.refund,(id))); }
}
contract DealEscrowTest is Test {
    DealEscrow e; TestCoin coin;
    address payer=address(0x10011); address payee=address(0x10022); address admin=address(0x10033); address treasury=address(0x10044); address stranger=address(0x10055);
    function setUp() public { coin=new TestCoin(); address[] memory tokens=new address[](1);tokens[0]=address(coin);e=new DealEscrow(admin,treasury,tokens);vm.deal(payer,100 ether);coin.mint(payer,100 ether);vm.prank(payer);coin.approve(address(e),type(uint256).max); }
    function fund(address token,uint256 amount,bytes32 salt,address recipient) internal returns(bytes32) { vm.prank(payer);return e.deposit{value:token==address(0)?amount:0}(salt,recipient,token,amount); }
    function testReleaseExactFeeAndNoSecondSpend() public {bytes32 id=fund(address(coin),10000,0,payee);vm.prank(payer);e.release(id);assertEq(coin.balanceOf(payee),9970);assertEq(coin.balanceOf(treasury),30);assertEq(e.liabilities(address(coin)),0);vm.prank(payer);vm.expectRevert(DealEscrow.InvalidState.selector);e.release(id);}
    function testRefundFullAndImmutableIdentity() public {bytes32 id=fund(address(0),1 ether,0,payee);vm.prank(payee);e.refund(id);assertEq(payer.balance,100 ether);assertEq(treasury.balance,0);vm.prank(payer);vm.expectRevert(DealEscrow.InvalidState.selector);e.deposit{value:1 ether}(0,stranger,address(0),1 ether);}
    function testAuthorityAndDisputeMatrix() public {bytes32 id=fund(address(0),1 ether,0,payee);vm.prank(admin);vm.expectRevert(DealEscrow.InvalidState.selector);e.resolve(id,true);vm.prank(stranger);vm.expectRevert(DealEscrow.Unauthorized.selector);e.release(id);vm.prank(payer);vm.expectRevert(DealEscrow.Unauthorized.selector);e.refund(id);vm.prank(stranger);vm.expectRevert(DealEscrow.Unauthorized.selector);e.dispute(id);vm.prank(payee);e.dispute(id);vm.prank(payer);vm.expectRevert(DealEscrow.InvalidState.selector);e.release(id);vm.prank(payee);vm.expectRevert(DealEscrow.Unauthorized.selector);e.resolve(id,true);vm.prank(admin);e.resolve(id,true);assertEq(payee.balance,997e15);assertEq(treasury.balance,3e15);}
    function testPayeeRefundWhileDisputedAndAdminRefund() public {bytes32 a=fund(address(0),1 ether,0,payee);bytes32 b=fund(address(0),1 ether,bytes32(uint256(1)),payee);vm.prank(payer);e.dispute(a);vm.prank(payee);e.refund(a);vm.prank(payee);e.dispute(b);vm.prank(admin);e.resolve(b,false);assertEq(payer.balance,100 ether);assertEq(address(e).balance,0);}
    function testPauseOnlyBlocksDeposits() public {bytes32 id=fund(address(0),1 ether,0,payee);vm.prank(stranger);vm.expectRevert(DealEscrow.Unauthorized.selector);e.setPaused(true);vm.prank(admin);e.setPaused(true);vm.prank(payer);vm.expectRevert();e.deposit{value:1 ether}(bytes32(uint256(1)),payee,address(0),1 ether);vm.prank(payer);e.release(id);assertEq(e.liabilities(address(0)),0);}
    function testOtherLiabilitiesStayFunded() public {bytes32 a=fund(address(0),1 ether,0,payee);fund(address(0),2 ether,bytes32(uint256(1)),stranger);vm.prank(payer);e.release(a);assertEq(e.liabilities(address(0)),2 ether);assertEq(address(e).balance,2 ether);}
    function testRejectedNativeTransferIsAtomic() public {bytes32 id=fund(address(0),1 ether,0,address(new RejectReceiver()));vm.prank(payer);vm.expectRevert(DealEscrow.TransferFailed.selector);e.release(id);(,,,,DealEscrow.Status status)=e.deals(id);assertEq(uint256(status),1);assertEq(e.liabilities(address(0)),1 ether);}
    function testReentrancyCannotRefundDuringRelease() public {ReenterReceiver receiver=new ReenterReceiver();bytes32 id=fund(address(0),1 ether,0,address(receiver));receiver.configure(e,id);vm.prank(payer);e.release(id);assertTrue(receiver.attempted());assertFalse(receiver.succeeded());assertEq(e.liabilities(address(0)),0);}
    function testRejectedTokenTransferAndTaxRejected() public {coin.failures(false,true);vm.prank(payer);vm.expectRevert(DealEscrow.IncorrectAmount.selector);e.deposit(0,payee,address(coin),10000);assertEq(e.liabilities(address(coin)),0);coin.failures(false,false);bytes32 id=fund(address(coin),10000,0,payee);coin.failures(true,false);vm.prank(payer);vm.expectRevert();e.release(id);assertEq(e.liabilities(address(coin)),10000);}
    function testWrongTokenAndAmount() public {TestCoin other=new TestCoin();vm.prank(payer);vm.expectRevert(DealEscrow.UnsupportedToken.selector);e.deposit(0,payee,address(other),1);vm.prank(payer);vm.expectRevert(DealEscrow.IncorrectAmount.selector);e.deposit{value:2}(0,payee,address(0),1);vm.prank(payer);vm.expectRevert(DealEscrow.InvalidDeal.selector);e.deposit(0,payee,address(0),0);}
    function testPayerSaltCannotBeSquatted() public {fund(address(0),1 ether,0,payee);vm.deal(stranger,1 ether);vm.prank(stranger);bytes32 other=e.deposit{value:1 ether}(0,payee,address(0),1 ether);assertTrue(other!=e.escrowId(payer,0));}
    function testMaximumTokenAmountCanBeReleasedWithoutOverflow() public {
        TestCoin huge=new TestCoin();
        address[] memory tokens=new address[](1);tokens[0]=address(huge);
        DealEscrow escrow=new DealEscrow(admin,treasury,tokens);
        huge.mint(payer,type(uint256).max);
        vm.startPrank(payer);huge.approve(address(escrow),type(uint256).max);
        bytes32 id=escrow.deposit(0,payee,address(huge),type(uint256).max);escrow.release(id);vm.stopPrank();
        assertEq(huge.balanceOf(payee)+huge.balanceOf(treasury),type(uint256).max);
        assertEq(escrow.liabilities(address(huge)),0);
    }
    function testFuzzFeeNoOverflow(uint256 amount) public view {assertEq(e.feeFor(amount),amount/10000*30+(amount%10000)*30/10000);}
    function testFuzzTokenConservation(uint128 raw,bool release_) public {uint256 amount=bound(uint256(raw),1,100 ether);bytes32 id=fund(address(coin),amount,0,payee);vm.prank(release_?payer:payee);if(release_) e.release(id);else e.refund(id);assertEq(coin.balanceOf(payer)+coin.balanceOf(payee)+coin.balanceOf(treasury)+coin.balanceOf(address(e)),100 ether);assertEq(e.liabilities(address(coin)),0);}
}
