// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/**
 * @title PaymentEscrow V2
 * @dev Dynamic payment escrow contract supporting context-based payments
 * @notice Users pay for features via dynamic links (plans, groups, products, campaigns, or custom)
 * 
 * Core Design Philosophy:
 * - Payments are tied to context (group/plan/link) enabling flexible feature unlocking
 * - No backward compatibility with V1 payForPlan() - fresh context-based approach
 * - Backend validates payment context and activates corresponding permissions
 */
contract PaymentEscrow is AccessControl, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using Address for address payable;

    // ============ Enums ============

    /**
     * @notice Context types for dynamic payments
     * @dev Each type maps to a different backend entity for feature unlocking
     */
    enum ContextType {
        PLAN,       // Subscription plan payment
        GROUP,      // Permission group payment
        PRODUCT,    // One-time product purchase
        CAMPAIGN,   // Promotional campaign payment
        CUSTOM      // Custom payment link
    }

    // ============ Roles ============

    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    // ============ State Variables ============

    /// @notice Mapping from token address to enabled status
    mapping(address => bool) public supportedTokens;

    /// @notice Total number of payments processed
    uint256 public totalPayments;

    /// @notice Minimum payment amount (to prevent dust attacks)
    uint256 public minPaymentAmount;

    /// @notice Maximum payment amount (safety limit)
    uint256 public maxPaymentAmount;

    /// @notice Daily spending limit per user (0 = unlimited)
    uint256 public dailyLimitPerUser;

    /// @notice Mapping of user => day => amount spent
    mapping(address => mapping(uint256 => uint256)) public userDailySpent;

    // ============ Events ============

    /**
     * @notice Emitted when a context-based payment is received
     * @param user Address of the user who paid
     * @param contextType Type of payment context (PLAN, GROUP, etc.)
     * @param contextId ID of the context entity (plan ID, group ID, etc.)
     * @param token Address of the ERC20 token used for payment
     * @param amount Amount paid (in token decimals)
     * @param timestamp Block timestamp of the payment
     * @param paymentId Unique sequential payment ID
     * @param linkHash Hash of the payment link for verification (optional, 0x0 if unused)
     */
    event PaymentWithContext(
        address indexed user,
        ContextType indexed contextType,
        uint256 indexed contextId,
        address token,
        uint256 amount,
        uint256 timestamp,
        uint256 paymentId,
        bytes32 linkHash
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

    /**
     * @notice Emitted when payment limits are updated
     * @param minAmount Minimum payment amount
     * @param maxAmount Maximum payment amount
     * @param dailyLimit Daily limit per user
     */
    event PaymentLimitsUpdated(uint256 minAmount, uint256 maxAmount, uint256 dailyLimit);

    // ============ Constructor ============

    constructor(address initialAdmin) {
        require(initialAdmin != address(0), "Invalid admin address");
        _grantRole(DEFAULT_ADMIN_ROLE, initialAdmin);
        _grantRole(MANAGER_ROLE, initialAdmin);
        // Default limits: no minimum, 1M tokens max, 100K daily limit
        maxPaymentAmount = 1_000_000 * 1e18;
        dailyLimitPerUser = 100_000 * 1e18;
        // Contract starts unpaused and ready to receive payments
    }

    // ============ User Functions ============

    /**
     * @notice Pay with context - primary payment function
     * @dev User must approve this contract to spend tokens before calling
     * @dev Backend is responsible for validating payment context and amount
     * @param contextType Type of payment context (PLAN, GROUP, PRODUCT, CAMPAIGN, CUSTOM)
     * @param contextId ID of the context entity (backend entity ID)
     * @param linkHash Hash of the payment link for verification (pass bytes32(0) if not using links)
     * @param token Address of the ERC20 token to pay with (USDT/USDC)
     * @param amount Amount to pay (in token decimals)
     */
    function payWithContext(
        ContextType contextType,
        uint256 contextId,
        bytes32 linkHash,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        // Validate inputs and limits
        _validatePayment(token, amount);

        // Increment payment counter (gas optimized)
        unchecked { ++totalPayments; }

        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        // Emit event for backend verification
        emit PaymentWithContext(
            msg.sender,
            contextType,
            contextId,
            token,
            amount,
            block.timestamp,
            totalPayments,
            linkHash
        );
    }

    /**
     * @notice Pay with context - payable version for MetaMask amount display
     * @dev Accepts native currency for display but refunds it
     * @dev User must approve this contract to spend tokens before calling
     * @param contextType Type of payment context
     * @param contextId ID of the context entity
     * @param linkHash Hash of the payment link for verification
     * @param token Address of the ERC20 token to pay with
     * @param amount Amount to pay in tokens
     */
    function payWithContextDisplay(
        ContextType contextType,
        uint256 contextId,
        bytes32 linkHash,
        address token,
        uint256 amount
    ) external payable whenNotPaused nonReentrant {
        // Validate inputs and limits
        _validatePayment(token, amount);

        // Increment payment counter (gas optimized)
        unchecked { ++totalPayments; }

        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        // Refund the native currency if any was sent (for MetaMask display only)
        if (msg.value > 0) {
            payable(msg.sender).sendValue(msg.value);
        }

        // Emit event for backend verification
        emit PaymentWithContext(
            msg.sender,
            contextType,
            contextId,
            token,
            amount,
            block.timestamp,
            totalPayments,
            linkHash
        );
    }

    /**
     * @notice Convenience function for plan payments
     * @param planId ID of the subscription plan
     * @param token Address of the ERC20 token to pay with
     * @param amount Amount to pay
     */
    function payForPlan(
        uint256 planId,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        _validatePayment(token, amount);

        unchecked { ++totalPayments; }
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit PaymentWithContext(
            msg.sender,
            ContextType.PLAN,
            planId,
            token,
            amount,
            block.timestamp,
            totalPayments,
            bytes32(0)
        );
    }

    /**
     * @notice Convenience function for group/permission payments
     * @param groupId ID of the permission group (as uint256)
     * @param token Address of the ERC20 token to pay with
     * @param amount Amount to pay
     */
    function payForGroup(
        uint256 groupId,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        _validatePayment(token, amount);

        unchecked { ++totalPayments; }
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit PaymentWithContext(
            msg.sender,
            ContextType.GROUP,
            groupId,
            token,
            amount,
            block.timestamp,
            totalPayments,
            bytes32(0)
        );
    }

    /**
     * @notice Pay via dynamic link
     * @dev linkHash is used by backend to verify payment link validity
     * @param contextType Type of payment context
     * @param contextId ID of the context entity
     * @param linkHash Keccak256 hash of the payment link slug
     * @param token Address of the ERC20 token to pay with
     * @param amount Amount to pay
     */
    function payViaLink(
        ContextType contextType,
        uint256 contextId,
        bytes32 linkHash,
        address token,
        uint256 amount
    ) external whenNotPaused nonReentrant {
        _validatePayment(token, amount);
        require(linkHash != bytes32(0), "Link hash required");

        unchecked { ++totalPayments; }
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit PaymentWithContext(
            msg.sender,
            contextType,
            contextId,
            token,
            amount,
            block.timestamp,
            totalPayments,
            linkHash
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
     * @notice Set payment limits for security
     * @param minAmount Minimum payment amount (0 = no minimum)
     * @param maxAmount Maximum payment amount (0 = unlimited)
     * @param dailyLimit Daily limit per user (0 = unlimited)
     */
    function setPaymentLimits(
        uint256 minAmount,
        uint256 maxAmount,
        uint256 dailyLimit
    ) external onlyRole(MANAGER_ROLE) {
        require(maxAmount == 0 || maxAmount > minAmount, "Max must be > min");
        
        minPaymentAmount = minAmount;
        maxPaymentAmount = maxAmount;
        dailyLimitPerUser = dailyLimit;

        emit PaymentLimitsUpdated(minAmount, maxAmount, dailyLimit);
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
     * @notice Get context type name as string
     * @param contextType The context type enum value
     * @return The context type name
     */
    function getContextTypeName(ContextType contextType) external pure returns (string memory) {
        if (contextType == ContextType.PLAN) return "PLAN";
        if (contextType == ContextType.GROUP) return "GROUP";
        if (contextType == ContextType.PRODUCT) return "PRODUCT";
        if (contextType == ContextType.CAMPAIGN) return "CAMPAIGN";
        if (contextType == ContextType.CUSTOM) return "CUSTOM";
        return "UNKNOWN";
    }

    /**
     * @notice Get current day number (for daily limit tracking)
     * @return Current day number since epoch
     */
    function getCurrentDay() public view returns (uint256) {
        return block.timestamp / 1 days;
    }

    /**
     * @notice Get user's remaining daily limit
     * @param user Address of the user
     * @return Remaining amount user can spend today (type(uint256).max if unlimited)
     */
    function getUserRemainingDailyLimit(address user) external view returns (uint256) {
        if (dailyLimitPerUser == 0) return type(uint256).max;
        
        uint256 today = getCurrentDay();
        uint256 spent = userDailySpent[user][today];
        
        if (spent >= dailyLimitPerUser) return 0;
        return dailyLimitPerUser - spent;
    }

    // ============ Internal Functions ============

    /**
     * @notice Validate payment against all limits
     * @param token Token address
     * @param amount Payment amount
     */
    function _validatePayment(address token, uint256 amount) internal {
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");
        require(amount >= minPaymentAmount, "Amount below minimum");
        require(maxPaymentAmount == 0 || amount <= maxPaymentAmount, "Amount exceeds maximum");

        // Check daily limit
        if (dailyLimitPerUser > 0) {
            uint256 today = getCurrentDay();
            uint256 newSpent = userDailySpent[msg.sender][today] + amount;
            require(newSpent <= dailyLimitPerUser, "Daily limit exceeded");
            userDailySpent[msg.sender][today] = newSpent;
        }
    }

    /**
     * @notice Allow contract to receive native currency
     */
    receive() external payable {}
}
