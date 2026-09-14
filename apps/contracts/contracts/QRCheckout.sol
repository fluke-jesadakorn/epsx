// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

/// Each invoice has a CREATE2 address. Ordinary token transfers need no approval
/// or connection to the checkout website. Anyone can forward funds, but neither
/// the caller nor the server can change their destination.
contract QRReceiver is ReentrancyGuard {
    using SafeERC20 for IERC20;
    address public immutable merchant;
    address public immutable treasury;
    constructor(address merchant_, address treasury_) {
        merchant = merchant_;
        treasury = treasury_;
    }
    function collect(address token) external nonReentrant {
        uint256 balance = IERC20(token).balanceOf(address(this));
        uint256 fee = Math.mulDiv(balance, 50, 10000);
        if (fee > 0) IERC20(token).safeTransfer(treasury, fee);
        if (balance > fee) IERC20(token).safeTransfer(merchant, balance - fee);
    }
}

contract QRCheckout is ReentrancyGuard {
    using SafeERC20 for IERC20;
    address public immutable treasury;
    uint256 public constant VERSION = 1;
    uint256 public constant FEE_BPS = 50;
    event QRRefunded(address indexed receiver, address indexed payer, address token, uint256 amount);
    event QRCollected(address indexed receiver);
    constructor(address treasury_) {
        require(treasury_ != address(0), "treasury");
        treasury = treasury_;
    }
    function invoiceSalt(address merchant, address token, uint256 amount, bytes32 salt) public pure returns (bytes32) {
        require(merchant != address(0) && token != address(0) && amount > 0, "terms");
        return keccak256(abi.encode(merchant, token, amount, salt));
    }
    function receiver(address merchant, address token, uint256 amount, bytes32 salt) public view returns (address) {
        bytes32 code = keccak256(abi.encodePacked(type(QRReceiver).creationCode, abi.encode(merchant, treasury)));
        return address(uint160(uint256(keccak256(abi.encodePacked(bytes1(0xff), address(this), invoiceSalt(merchant, token, amount, salt), code)))));
    }
    function collect(address merchant, address token, uint256 amount, bytes32 salt) external nonReentrant {
        address target = receiver(merchant, token, amount, salt);
        if (target.code.length == 0) {
            new QRReceiver{salt: invoiceSalt(merchant, token, amount, salt)}(merchant, treasury);
        }
        QRReceiver(target).collect(token);
        emit QRCollected(target);
    }
    /// Merchant-funded full refund; the indexer checks the recipient against the
    /// verified deposit. A caller can only spend its own allowance.
    function refundTransfer(address token, uint256 amount, bytes32 salt, address payer) external nonReentrant {
        require(payer != address(0) && payer != msg.sender, "payer");
        IERC20(token).safeTransferFrom(msg.sender, payer, amount);
        emit QRRefunded(receiver(msg.sender, token, amount, salt), payer, token, amount);
    }
}
