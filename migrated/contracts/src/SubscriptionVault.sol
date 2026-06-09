// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title SubscriptionVault
 * @notice Per-merchant subscription vault.
 *         Subscribers deposit USDC/USDT to subscribe; merchants claim accumulated fees.
 *         Uses interval-based billing with a grace period.
 */
contract SubscriptionVault is Ownable, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;

    enum Status { Active, Cancelled, Expired }

    struct Plan {
        address merchant;
        address token;
        uint256 amountPerPeriod;
        uint64 periodSeconds; // 1 day, 1 week, 1 month
        uint8 gracePeriods;   // missed periods before expiry
        bool active;
        uint64 createdAt;
    }

    struct Subscription {
        address subscriber;
        uint64 planId;
        uint64 lastPaidAt;
        uint8 periodsPaid;
        Status status;
        uint256 totalPaid;
        uint64 startedAt;
    }

    mapping(uint64 => Plan) public plans;
    mapping(uint64 => Subscription) public subscriptions;
    mapping(address => uint256) public merchantBalances;

    uint64 public nextPlanId = 1;
    uint64 public nextSubId = 1;

    event PlanCreated(uint64 indexed planId, address indexed merchant, address token, uint256 amountPerPeriod, uint64 periodSeconds);
    event Subscribed(uint64 indexed subId, uint64 indexed planId, address indexed subscriber);
    event Charged(uint64 indexed subId, uint256 amount, uint64 chargedAt, uint8 periodsCount);
    event Cancelled(uint64 indexed subId);
    event Withdrawn(address indexed merchant, address indexed token, uint256 amount);
    event Expired(uint64 indexed subId);

    error PlanNotFound();
    error PlanInactive();
    error SubscriptionNotFound();
    error NotSubscriber();
    error ZeroAmount();
    error ZeroAddress();
    error NothingDue();

    constructor() Ownable(msg.sender) {}

    function createPlan(
        address token,
        uint256 amountPerPeriod,
        uint64 periodSeconds,
        uint8 gracePeriods
    ) external whenNotPaused returns (uint64 planId) {
        if (token == address(0)) revert ZeroAddress();
        if (amountPerPeriod == 0) revert ZeroAmount();
        if (periodSeconds == 0) revert ZeroAmount();
        if (gracePeriods > 12) revert PlanInactive();

        planId = nextPlanId++;
        plans[planId] = Plan({
            merchant: msg.sender,
            token: token,
            amountPerPeriod: amountPerPeriod,
            periodSeconds: periodSeconds,
            gracePeriods: gracePeriods,
            active: true,
            createdAt: uint64(block.timestamp)
        });

        emit PlanCreated(planId, msg.sender, token, amountPerPeriod, periodSeconds);
    }

    function subscribe(uint64 planId) external whenNotPaused nonReentrant returns (uint64 subId) {
        Plan storage p = plans[planId];
        if (p.merchant == address(0)) revert PlanNotFound();
        if (!p.active) revert PlanInactive();

        subId = nextSubId++;
        subscriptions[subId] = Subscription({
            subscriber: msg.sender,
            planId: planId,
            lastPaidAt: uint64(block.timestamp),
            periodsPaid: 0,
            status: Status.Active,
            totalPaid: 0,
            startedAt: uint64(block.timestamp)
        });

        emit Subscribed(subId, planId, msg.sender);
    }

    function charge(uint64 subId, uint8 periodsToCharge) external nonReentrant {
        Subscription storage s = subscriptions[subId];
        if (s.subscriber == address(0)) revert SubscriptionNotFound();
        if (s.status != Status.Active) revert SubscriptionNotFound();
        if (periodsToCharge == 0) revert ZeroAmount();

        Plan storage p = plans[s.planId];
        if (!p.active) revert PlanInactive();

        uint256 total = p.amountPerPeriod * periodsToCharge;

        IERC20(p.token).safeTransferFrom(s.subscriber, address(this), total);

        merchantBalances[p.merchant] += total;
        s.lastPaidAt = uint64(block.timestamp);
        s.periodsPaid += periodsToCharge;
        s.totalPaid += total;

        emit Charged(subId, total, s.lastPaidAt, periodsToCharge);
    }

    function cancel(uint64 subId) external {
        Subscription storage s = subscriptions[subId];
        if (s.subscriber == address(0)) revert SubscriptionNotFound();
        if (msg.sender != s.subscriber) revert NotSubscriber();
        if (s.status != Status.Active) revert SubscriptionNotFound();

        s.status = Status.Cancelled;
        emit Cancelled(subId);
    }

    function checkAndExpire(uint64 subId) external {
        Subscription storage s = subscriptions[subId];
        if (s.subscriber == address(0)) revert SubscriptionNotFound();
        if (s.status != Status.Active) revert SubscriptionNotFound();

        Plan storage p = plans[s.planId];
        uint64 since = uint64(block.timestamp) - s.lastPaidAt;
        uint64 maxMissed = uint64(p.gracePeriods) * p.periodSeconds;

        if (since > maxMissed) {
            s.status = Status.Expired;
            emit Expired(subId);
        }
    }

    function withdraw(address token) external nonReentrant {
        uint256 amount = merchantBalances[msg.sender];
        if (amount == 0) revert NothingDue();
        merchantBalances[msg.sender] = 0;
        IERC20(token).safeTransfer(msg.sender, amount);
        emit Withdrawn(msg.sender, token, amount);
    }

    function getPlan(uint64 planId) external view returns (Plan memory) {
        return plans[planId];
    }

    function getSubscription(uint64 subId) external view returns (Subscription memory) {
        return subscriptions[subId];
    }

    function setPlanActive(uint64 planId, bool active) external {
        Plan storage p = plans[planId];
        if (p.merchant == address(0)) revert PlanNotFound();
        require(msg.sender == p.merchant || msg.sender == owner(), "Not authorized");
        p.active = active;
    }

    function pause() external onlyOwner { _pause(); }
    function unpause() external onlyOwner { _unpause(); }
}
