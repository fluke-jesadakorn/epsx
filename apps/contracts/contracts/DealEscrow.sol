// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

/// @notice EPSX Pay v1. Separate deployment from the legacy subscription contract.
/// @dev No proxy, arbitrary withdrawals, automatic release, or backend signer.
/// Only conventional, non-rebasing, non-taxed allowlisted tokens are supported.
contract DealEscrow is ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;
    enum Status { None, Active, Released, Refunded, Disputed }
    struct Deal { address payer; address payee; address token; uint256 amount; Status status; }
    uint256 public constant FEE_BPS = 30;
    uint256 public constant VERSION = 1;
    address public immutable arbiter;
    address public immutable treasury;
    mapping(address => bool) public supportedTokens;
    mapping(bytes32 => Deal) public deals;
    mapping(address => uint256) public liabilities;

    error Unauthorized();
    error InvalidDeal();
    error InvalidState();
    error UnsupportedToken();
    error IncorrectAmount();
    error TransferFailed();
    event Deposited(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount);
    event Released(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount, uint256 fee);
    event Refunded(bytes32 indexed id, address indexed payer, address indexed payee, address token, uint256 amount);
    event Disputed(bytes32 indexed id, address indexed actor);

    constructor(address adminWallet, address feeRecipient, address[] memory tokens) {
        if (adminWallet == address(0) || feeRecipient == address(0) || feeRecipient == address(this)) revert InvalidDeal();
        arbiter = adminWallet;
        treasury = feeRecipient;
        supportedTokens[address(0)] = true;
        for (uint256 i; i < tokens.length; ++i) {
            if (tokens[i] == address(0) || tokens[i].code.length == 0) revert UnsupportedToken();
            supportedTokens[tokens[i]] = true;
        }
    }
    modifier onlyArbiter() { if (msg.sender != arbiter) revert Unauthorized(); _; }
    function setPaused(bool paused_) external onlyArbiter { if (paused_) _pause(); else _unpause(); }
    /// @dev Binding the salt to the sender prevents another wallet from squatting an intent ID.
    function escrowId(address payer, bytes32 salt) public view returns (bytes32) {
        return keccak256(abi.encode(block.chainid, address(this), payer, salt));
    }
    function feeFor(uint256 amount) public pure returns (uint256) { return Math.mulDiv(amount, FEE_BPS, 10000); }
    function deposit(bytes32 salt, address payee, address token, uint256 amount) external payable nonReentrant whenNotPaused returns (bytes32 id) {
        if (amount == 0 || payee == address(0) || payee == msg.sender || payee == address(this)) revert InvalidDeal();
        if (!supportedTokens[token]) revert UnsupportedToken();
        id = escrowId(msg.sender, salt);
        if (deals[id].status != Status.None) revert InvalidState();
        deals[id] = Deal(msg.sender, payee, token, amount, Status.Active);
        liabilities[token] += amount;
        if (token == address(0)) {
            if (msg.value != amount) revert IncorrectAmount();
        } else {
            if (msg.value != 0) revert IncorrectAmount();
            uint256 before_ = IERC20(token).balanceOf(address(this));
            IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
            if (IERC20(token).balanceOf(address(this)) - before_ != amount) revert IncorrectAmount();
        }
        emit Deposited(id, msg.sender, payee, token, amount);
    }
    function release(bytes32 id) external nonReentrant {
        Deal storage d = deals[id];
        if (msg.sender != d.payer) revert Unauthorized();
        if (d.status != Status.Active) revert InvalidState();
        settle(id, d, true);
    }
    function refund(bytes32 id) external nonReentrant {
        Deal storage d = deals[id];
        if (msg.sender != d.payee) revert Unauthorized();
        if (d.status != Status.Active && d.status != Status.Disputed) revert InvalidState();
        settle(id, d, false);
    }
    function dispute(bytes32 id) external nonReentrant {
        Deal storage d = deals[id];
        if (msg.sender != d.payer && msg.sender != d.payee) revert Unauthorized();
        if (d.status != Status.Active) revert InvalidState();
        d.status = Status.Disputed;
        emit Disputed(id, msg.sender);
    }
    function resolve(bytes32 id, bool toPayee) external onlyArbiter nonReentrant {
        Deal storage d = deals[id];
        if (d.status != Status.Disputed) revert InvalidState();
        settle(id, d, toPayee);
    }
    function settle(bytes32 id, Deal storage d, bool toPayee) private {
        d.status = toPayee ? Status.Released : Status.Refunded;
        liabilities[d.token] -= d.amount;
        if (toPayee) {
            uint256 fee = feeFor(d.amount);
            transfer(d.token, d.payee, d.amount - fee);
            if (fee != 0) transfer(d.token, treasury, fee);
            emit Released(id, d.payer, d.payee, d.token, d.amount, fee);
        } else {
            transfer(d.token, d.payer, d.amount);
            emit Refunded(id, d.payer, d.payee, d.token, d.amount);
        }
    }
    function transfer(address token, address recipient, uint256 amount) private {
        if (token == address(0)) {
            (bool success,) = recipient.call{value: amount}("");
            if (!success) revert TransferFailed();
        } else {
            uint256 before_ = IERC20(token).balanceOf(recipient);
            IERC20(token).safeTransfer(recipient, amount);
            if (IERC20(token).balanceOf(recipient) - before_ != amount) revert IncorrectAmount();
        }
    }
}
