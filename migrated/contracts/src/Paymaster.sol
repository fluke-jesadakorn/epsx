// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title Paymaster
 * @notice ERC-4337 style paymaster that sponsors gas for users paying with USDC/USDT.
 *         Users pre-deposit stablecoins; the paymaster converts these to native gas.
 *
 * Note: This is a simplified paymaster. A full implementation would use
 * EntryPoint.handleOps() and verify UserOperations via off-chain signature.
 */
contract Paymaster is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    address public entryPoint;
    address public feeRecipient;
    uint256 public gasPriceMarkupBps; // e.g. 100 = 1% markup

    mapping(address => uint256) public deposits; // user => stablecoin balance
    mapping(address => uint256) public sponsorAllowance; // user => max gas to sponsor

    event Deposited(address indexed user, address indexed token, uint256 amount);
    event Sponsored(address indexed user, uint256 gasAmount, uint256 stableAmount, address indexed token);
    event Withdrawn(address indexed user, address indexed token, uint256 amount);

    error ZeroAddress();
    error ZeroAmount();
    error InsufficientDeposit();
    error NotEntryPoint();
    error TransferFailed();

    constructor(address _entryPoint, address _feeRecipient) Ownable(msg.sender) {
        if (_entryPoint == address(0)) revert ZeroAddress();
        if (_feeRecipient == address(0)) revert ZeroAddress();
        entryPoint = _entryPoint;
        feeRecipient = _feeRecipient;
        gasPriceMarkupBps = 100; // 1% default
    }

    function deposit(address token, uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        deposits[msg.sender] += amount;
        emit Deposited(msg.sender, token, amount);
    }

    function withdraw(address token, uint256 amount) external nonReentrant {
        if (amount == 0) revert ZeroAmount();
        if (deposits[msg.sender] < amount) revert InsufficientDeposit();
        deposits[msg.sender] -= amount;
        IERC20(token).safeTransfer(msg.sender, amount);
        emit Withdrawn(msg.sender, token, amount);
    }

    /**
     * @notice Sponsor a user operation by debiting their stablecoin deposit.
     *         Called by EntryPoint after the UserOp is processed.
     */
    function sponsorUserOp(
        address user,
        address token,
        uint256 gasUsed,
        uint256 gasPrice
    ) external nonReentrant returns (uint256 stableAmount) {
        if (msg.sender != entryPoint) revert NotEntryPoint();
        if (deposits[user] == 0) revert InsufficientDeposit();

        uint256 nativeCost = gasUsed * gasPrice;
        stableAmount = (nativeCost * (10000 + gasPriceMarkupBps)) / 10000;

        if (deposits[user] < stableAmount) revert InsufficientDeposit();
        deposits[user] -= stableAmount;

        IERC20(token).safeTransfer(feeRecipient, stableAmount);
        emit Sponsored(user, gasUsed, stableAmount, token);
    }

    function setGasMarkup(uint256 _gasPriceMarkupBps) external onlyOwner {
        require(_gasPriceMarkupBps <= 1000, "max 10%");
        gasPriceMarkupBps = _gasPriceMarkupBps;
    }

    function setEntryPoint(address _entryPoint) external onlyOwner {
        if (_entryPoint == address(0)) revert ZeroAddress();
        entryPoint = _entryPoint;
    }

    function setFeeRecipient(address _feeRecipient) external onlyOwner {
        if (_feeRecipient == address(0)) revert ZeroAddress();
        feeRecipient = _feeRecipient;
    }
}
