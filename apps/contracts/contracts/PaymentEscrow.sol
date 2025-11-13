// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title PaymentEscrow
 * @dev Escrow contract for subscription plan payments on BSC
 * @notice Users pay for subscription plans with USDT/USDC, contract emits events for backend verification
 */
contract PaymentEscrow is Ownable, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ============ State Variables ============

    /// @notice Mapping from plan ID to price in USD (6 decimals)
    mapping(uint256 => uint256) public planPrices;

    /// @notice Mapping from token address to enabled status
    mapping(address => bool) public supportedTokens;

    /// @notice Total number of payments processed
    uint256 public totalPayments;

    /// @notice Mapping to track processed payments and prevent duplicates
    mapping(bytes32 => bool) public processedPayments;

    // ============ Events ============

    /**
     * @notice Emitted when a payment is successfully received
     * @param user Address of the user who paid
     * @param planId ID of the subscription plan
     * @param token Address of the ERC20 token used for payment
     * @param amount Amount paid (in token decimals)
     * @param timestamp Block timestamp of the payment
     * @param paymentId Unique sequential payment ID
     */
    event PaymentReceived(
        address indexed user,
        uint256 indexed planId,
        address indexed token,
        uint256 amount,
        uint256 timestamp,
        uint256 paymentId
    );

    /**
     * @notice Emitted when a plan price is updated
     * @param planId ID of the subscription plan
     * @param oldPrice Previous price
     * @param newPrice New price
     */
    event PlanPriceUpdated(
        uint256 indexed planId,
        uint256 oldPrice,
        uint256 newPrice
    );

    /**
     * @notice Emitted when a token's supported status is updated
     * @param token Address of the token
     * @param enabled Whether the token is now enabled
     */
    event TokenStatusUpdated(address indexed token, bool enabled);

    /**
     * @notice Emitted when funds are withdrawn
     * @param token Address of the token withdrawn
     * @param amount Amount withdrawn
     * @param recipient Address receiving the funds
     */
    event FundsWithdrawn(
        address indexed token,
        uint256 amount,
        address indexed recipient
    );

    // ============ Constructor ============

    constructor(address initialOwner) Ownable(initialOwner) {
        // Contract starts unpaused and ready to receive payments
    }

    // ============ User Functions ============

    /**
     * @notice Pay for a subscription plan
     * @dev User must approve this contract to spend tokens before calling
     * @param planId ID of the subscription plan
     * @param token Address of the ERC20 token to pay with (USDT/USDC)
     * @param amount Amount to pay (must match plan price)
     */
    function payForPlan(
        uint256 planId,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        // Validate inputs
        require(supportedTokens[token], "Token not supported");
        require(planPrices[planId] > 0, "Invalid plan ID");
        require(amount == planPrices[planId], "Incorrect payment amount");

        // Generate unique payment hash to prevent duplicates
        bytes32 paymentHash = keccak256(
            abi.encodePacked(msg.sender, planId, token, amount, block.timestamp)
        );
        require(!processedPayments[paymentHash], "Duplicate payment");

        // Mark payment as processed
        processedPayments[paymentHash] = true;

        // Increment payment counter
        totalPayments++;

        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        // Emit event for backend verification
        emit PaymentReceived(
            msg.sender,
            planId,
            token,
            amount,
            block.timestamp,
            totalPayments
        );
    }

    // ============ Admin Functions ============

    /**
     * @notice Set the price for a subscription plan
     * @param planId ID of the subscription plan
     * @param price Price in USD (6 decimals, e.g., 59000000 = $59)
     */
    function setPlanPrice(uint256 planId, uint256 price) external onlyOwner {
        require(planId > 0, "Invalid plan ID");
        require(price > 0, "Price must be greater than 0");

        uint256 oldPrice = planPrices[planId];
        planPrices[planId] = price;

        emit PlanPriceUpdated(planId, oldPrice, price);
    }

    /**
     * @notice Enable or disable a token for payments
     * @param token Address of the ERC20 token
     * @param enabled Whether to enable or disable the token
     */
    function setTokenEnabled(address token, bool enabled) external onlyOwner {
        require(token != address(0), "Invalid token address");

        supportedTokens[token] = enabled;

        emit TokenStatusUpdated(token, enabled);
    }

    /**
     * @notice Withdraw collected funds to a recipient
     * @param token Address of the ERC20 token to withdraw
     * @param amount Amount to withdraw
     * @param recipient Address to receive the funds
     */
    function withdrawFunds(
        address token,
        uint256 amount,
        address recipient
    ) external onlyOwner nonReentrant {
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be greater than 0");

        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance >= amount, "Insufficient contract balance");

        IERC20(token).safeTransfer(recipient, amount);

        emit FundsWithdrawn(token, amount, recipient);
    }

    /**
     * @notice Pause the contract (emergency stop)
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    // ============ View Functions ============

    /**
     * @notice Get the price of a subscription plan
     * @param planId ID of the subscription plan
     * @return Price in USD (6 decimals)
     */
    function getPlanPrice(uint256 planId) external view returns (uint256) {
        return planPrices[planId];
    }

    /**
     * @notice Check if a token is supported for payments
     * @param token Address of the ERC20 token
     * @return Whether the token is supported
     */
    function isTokenSupported(address token) external view returns (bool) {
        return supportedTokens[token];
    }

    /**
     * @notice Get the contract's balance of a specific token
     * @param token Address of the ERC20 token
     * @return Balance of the token
     */
    function getTokenBalance(address token) external view returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }

    /**
     * @notice Get the total number of payments processed
     * @return Total payment count
     */
    function getTotalPayments() external view returns (uint256) {
        return totalPayments;
    }
}
