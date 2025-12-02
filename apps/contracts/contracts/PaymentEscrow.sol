// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title PaymentEscrow
 * @dev Escrow contract for subscription plan payments on BSC
 * @notice Users pay for subscription plans with USDT/USDC, contract emits events for backend verification
 */
contract PaymentEscrow is AccessControl, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ============ Roles ============

    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    // ============ State Variables ============

    /// @notice Mapping from token address to enabled status
    mapping(address => bool) public supportedTokens;

    /// @notice Total number of payments processed
    uint256 public totalPayments;

    // ============ Events ============

    /**
     * @notice Emitted when a payment is successfully received
     * @param user Address of the user who paid
     * @param planId ID of the subscription plan (for backend tracking)
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

    /**
     * @notice Emitted when native funds (BNB/ETH) are withdrawn
     * @param amount Amount withdrawn
     * @param recipient Address receiving the funds
     */
    event NativeFundsWithdrawn(uint256 amount, address indexed recipient);

    // ============ Constructor ============

    constructor(address initialAdmin) {
        _grantRole(DEFAULT_ADMIN_ROLE, initialAdmin);
        _grantRole(MANAGER_ROLE, initialAdmin);
        // Contract starts unpaused and ready to receive payments
    }

    // ============ User Functions ============

    /**
     * @notice Pay for a subscription plan
     * @dev User must approve this contract to spend tokens before calling
     * @dev Contract acts as payment gateway - does not validate plan or amount
     * @dev Backend is responsible for validating payment amounts match plan prices
     * @param planId ID of the subscription plan (logged for backend tracking)
     * @param token Address of the ERC20 token to pay with (USDT/USDC)
     * @param amount Amount to pay (any amount accepted)
     */
    function payForPlan(
        uint256 planId,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        // Validate inputs
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");

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

    /**
     * @notice Pay for a subscription plan using direct transfer
     * @dev Optimized for better MetaMask display - functionally identical to payForPlan
     * @dev User must approve this contract to spend tokens before calling
     * @dev Contract acts as payment gateway - does not validate plan or amount
     * @dev Backend is responsible for validating payment amounts match plan prices
     * @param planId ID of the subscription plan (logged for backend tracking)
     * @param token Address of the ERC20 token to pay with (USDT/USDC)
     * @param amount Amount to pay (any amount accepted)
     */
    function payWithTransfer(
        uint256 planId,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        // Validate inputs
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");

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

    /**
     * @notice Pay for a subscription plan with explicit amount display for MetaMask
     * @dev This function accepts native currency as reference but doesn't actually use it
     * @dev The native currency amount is just for MetaMask display purposes
     * @dev User must approve this contract to spend tokens before calling
     * @dev Contract acts as payment gateway - does not validate plan or amount
     * @dev Backend is responsible for validating payment amounts match plan prices
     * @param planId ID of the subscription plan (logged for backend tracking)
     * @param token Address of the ERC20 token to pay with (USDT/USDC)
     * @param amount Amount to pay in tokens (any amount accepted)
     */
    function payWithAmountDisplay(
        uint256 planId,
        address token,
        uint256 amount
    ) external payable whenNotPaused nonReentrant {
        // Validate inputs
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");

        // Note: msg.value is ignored - it's just for MetaMask display
        // The actual payment happens with ERC20 tokens

        // Increment payment counter
        totalPayments++;

        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        // Refund the native currency if any was sent
        if (msg.value > 0) {
            (bool success, ) = payable(msg.sender).call{value: msg.value}("");
            require(success, "Refund failed");
        }

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
     * @notice Enable or disable a token for payments
     * @param token Address of the ERC20 token
     * @param enabled Whether to enable or disable the token
     */
    function setTokenEnabled(address token, bool enabled) external onlyRole(MANAGER_ROLE) {
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
    ) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be greater than 0");

        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance >= amount, "Insufficient contract balance");

        IERC20(token).safeTransfer(recipient, amount);

        emit FundsWithdrawn(token, amount, recipient);
    }

    /**
     * @notice Withdraw native funds (BNB/ETH) to a recipient
     * @param amount Amount to withdraw
     * @param recipient Address to receive the funds
     */
    function withdrawNative(
        uint256 amount,
        address payable recipient
    ) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be greater than 0");
        require(address(this).balance >= amount, "Insufficient contract balance");

        (bool success, ) = recipient.call{value: amount}("");
        require(success, "Transfer failed");

        emit NativeFundsWithdrawn(amount, recipient);
    }

    /**
     * @notice Pause the contract (emergency stop)
     */
    function pause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _pause();
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _unpause();
    }

    // ============ View Functions ============

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

    /**
     * @notice Allow contract to receive native currency
     */
    receive() external payable {}
}
