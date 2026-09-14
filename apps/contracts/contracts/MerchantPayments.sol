// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

/// @notice Fixed-term payments. No proxy, off-chain signer, arbitrary withdrawal or auto release.
/// @dev Only conventional, non-rebasing, non-taxed tokens are supported.
abstract contract MerchantPayments is ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;
    enum Status { None, Active, Paid, Disputed, Refunded }
    struct Payment { address payer; address payee; address token; uint256 amount; Status status; }
    address public immutable arbiter;
    address public immutable treasury;
    uint256 public immutable FEE_BPS;
    uint256 public immutable VERSION;
    bool public immutable IS_ESCROW;
    mapping(address => bool) public supportedTokens;
    mapping(bytes32 => Payment) public payments;
    mapping(address => uint256) public liabilities;

    error Unauthorized();
    error InvalidTerms();
    error InvalidState();
    error UnsupportedToken();
    error IncorrectAmount();
    error TransferFailed();
    event Paid(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount, uint256 fee);
    event Funded(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount);
    event Released(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount, uint256 fee);
    event Refunded(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount);
    event Disputed(bytes32 indexed id, address indexed actor);

    constructor(address admin, address recipient, address[] memory tokens, bool escrow) {
        if (admin == address(0) || recipient == address(0) || recipient == address(this)) revert InvalidTerms();
        arbiter = admin; treasury = recipient; IS_ESCROW = escrow;
        FEE_BPS = escrow ? 100 : 50; VERSION = escrow ? 2 : 1;
        supportedTokens[address(0)] = true;
        for (uint256 i; i < tokens.length; ++i) {
            if (tokens[i] == address(0) || tokens[i].code.length == 0) revert UnsupportedToken();
            supportedTokens[tokens[i]] = true;
        }
    }
    function feeFor(uint256 amount) public view returns (uint256) { return Math.mulDiv(amount, FEE_BPS, 10000); }
    function paymentId(address payer, address payee, address token, uint256 amount, uint256 deadline, bytes32 salt) public view returns (bytes32) {
        return keccak256(abi.encode(block.chainid, address(this), payer, payee, token, amount, deadline, salt));
    }
    function setPaused(bool value) external {
        if (msg.sender != arbiter) revert Unauthorized();
        if (value) _pause(); else _unpause();
    }
    function accept(address payee, address token, uint256 amount, uint256 deadline, bytes32 salt) internal whenNotPaused returns (bytes32 id) {
        if (amount == 0 || deadline < block.timestamp || payee == address(0) || payee == msg.sender || payee == address(this)) revert InvalidTerms();
        if (!supportedTokens[token]) revert UnsupportedToken();
        id = paymentId(msg.sender, payee, token, amount, deadline, salt);
        if (payments[id].status != Status.None) revert InvalidState();
        payments[id] = Payment(msg.sender, payee, token, amount, IS_ESCROW ? Status.Active : Status.Paid);
        collect(token, msg.sender, amount);
        if (IS_ESCROW) {
            liabilities[token] += amount;
            emit Funded(id, msg.sender, payee, token, amount);
        } else {
            uint256 fee = feeFor(amount);
            transfer(token, payee, amount - fee);
            transfer(token, treasury, fee);
            emit Paid(id, msg.sender, payee, token, amount, fee);
        }
    }
    function release(bytes32 id) external nonReentrant {
        Payment storage p = payments[id];
        if (msg.sender != p.payer) revert Unauthorized();
        if (!IS_ESCROW || p.status != Status.Active) revert InvalidState();
        settle(id, p, true);
    }
    /// @notice Direct refunds are newly funded by the original merchant; the original fee is retained.
    function refund(bytes32 id) external payable nonReentrant {
        Payment storage p = payments[id];
        if (msg.sender != p.payee) revert Unauthorized();
        if (IS_ESCROW) {
            if (msg.value != 0) revert IncorrectAmount();
            if (p.status != Status.Active && p.status != Status.Disputed) revert InvalidState();
            settle(id, p, false);
        } else {
            if (p.status != Status.Paid) revert InvalidState();
            p.status = Status.Refunded;
            collect(p.token, msg.sender, p.amount);
            transfer(p.token, p.payer, p.amount);
            emit Refunded(id, p.payer, p.payee, p.token, p.amount);
        }
    }
    function dispute(bytes32 id) external nonReentrant {
        Payment storage p = payments[id];
        if (msg.sender != p.payer && msg.sender != p.payee) revert Unauthorized();
        if (!IS_ESCROW || p.status != Status.Active) revert InvalidState();
        p.status = Status.Disputed;
        emit Disputed(id, msg.sender);
    }
    function resolve(bytes32 id, bool toPayee) external nonReentrant {
        if (msg.sender != arbiter) revert Unauthorized();
        Payment storage p = payments[id];
        if (!IS_ESCROW || p.status != Status.Disputed) revert InvalidState();
        settle(id, p, toPayee);
    }
    function settle(bytes32 id, Payment storage p, bool toPayee) private {
        p.status = toPayee ? Status.Paid : Status.Refunded;
        liabilities[p.token] -= p.amount;
        if (toPayee) {
            uint256 fee = feeFor(p.amount);
            transfer(p.token, p.payee, p.amount - fee);
            transfer(p.token, treasury, fee);
            emit Released(id, p.payer, p.payee, p.token, p.amount, fee);
        } else {
            transfer(p.token, p.payer, p.amount);
            emit Refunded(id, p.payer, p.payee, p.token, p.amount);
        }
    }
    function collect(address token, address from, uint256 amount) private {
        if (token == address(0)) {
            if (msg.value != amount) revert IncorrectAmount();
        } else {
            if (msg.value != 0) revert IncorrectAmount();
            uint256 before_ = IERC20(token).balanceOf(address(this));
            IERC20(token).safeTransferFrom(from, address(this), amount);
            if (IERC20(token).balanceOf(address(this)) - before_ != amount) revert IncorrectAmount();
        }
    }
    function transfer(address token, address to, uint256 amount) private {
        if (amount == 0) return;
        if (token == address(0)) {
            (bool ok,) = payable(to).call{value: amount}("");
            if (!ok) revert TransferFailed();
        } else {
            uint256 before_ = IERC20(token).balanceOf(to);
            IERC20(token).safeTransfer(to, amount);
            if (IERC20(token).balanceOf(to) - before_ != amount) revert IncorrectAmount();
        }
    }
}

contract DirectPayments is MerchantPayments {
    constructor(address admin, address recipient, address[] memory tokens) MerchantPayments(admin, recipient, tokens, false) {}
    function pay(address payee, address token, uint256 amount, uint256 deadline, bytes32 salt) external payable nonReentrant returns (bytes32) {
        return accept(payee, token, amount, deadline, salt);
    }
}

contract MerchantEscrow is MerchantPayments {
    constructor(address admin, address recipient, address[] memory tokens) MerchantPayments(admin, recipient, tokens, true) {}
    function deposit(address payee, address token, uint256 amount, uint256 deadline, bytes32 salt) external payable nonReentrant returns (bytes32) {
        return accept(payee, token, amount, deadline, salt);
    }
}
