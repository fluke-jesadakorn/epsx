// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title PaymentEscrow
 * @notice Hold stablecoin payments in escrow between buyer and seller.
 *         Releases funds when conditions are met, or refunds on dispute.
 */
contract PaymentEscrow is Ownable, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;

    enum Status { Active, Released, Refunded, Disputed }

    struct Escrow {
        address buyer;
        address seller;
        address token;
        uint256 amount;
        uint256 fee;
        bytes32 orderId;
        Status status;
        uint64 createdAt;
        uint64 releasedAt;
    }

    mapping(bytes32 => Escrow) public escrows;
    mapping(address => uint256) public sellerBalances;

    address public feeRecipient;
    uint16 public feeBps; // basis points (e.g. 30 = 0.30%)

    event EscrowCreated(
        bytes32 indexed escrowId,
        address indexed buyer,
        address indexed seller,
        address token,
        uint256 amount,
        bytes32 orderId
    );
    event EscrowReleased(bytes32 indexed escrowId, address indexed seller, uint256 amount, uint256 fee);
    event EscrowRefunded(bytes32 indexed escrowId, address indexed buyer, uint256 amount);
    event EscrowDisputed(bytes32 indexed escrowId, address indexed by);
    event FeeWithdrawn(address indexed token, address indexed to, uint256 amount);

    error EscrowNotFound();
    error EscrowNotActive();
    error NotBuyerOrSeller();
    error NotBuyer();
    error ZeroAmount();
    error ZeroAddress();
    error FeeTooHigh();

    constructor(address _feeRecipient, uint16 _feeBps) Ownable(msg.sender) {
        if (_feeRecipient == address(0)) revert ZeroAddress();
        if (_feeBps > 1000) revert FeeTooHigh(); // max 10%
        feeRecipient = _feeRecipient;
        feeBps = _feeBps;
    }

    function createEscrow(
        address seller,
        address token,
        uint256 amount,
        bytes32 orderId
    ) external whenNotPaused nonReentrant returns (bytes32 escrowId) {
        if (seller == address(0)) revert ZeroAddress();
        if (token == address(0)) revert ZeroAddress();
        if (amount == 0) revert ZeroAmount();

        escrowId = keccak256(abi.encodePacked(msg.sender, seller, token, amount, orderId, block.timestamp));

        escrows[escrowId] = Escrow({
            buyer: msg.sender,
            seller: seller,
            token: token,
            amount: amount,
            fee: 0,
            orderId: orderId,
            status: Status.Active,
            createdAt: uint64(block.timestamp),
            releasedAt: 0
        });

        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit EscrowCreated(escrowId, msg.sender, seller, token, amount, orderId);
    }

    function releaseEscrow(bytes32 escrowId) external nonReentrant {
        Escrow storage e = escrows[escrowId];
        if (e.buyer == address(0)) revert EscrowNotFound();
        if (e.status != Status.Active) revert EscrowNotActive();
        if (msg.sender != e.buyer && msg.sender != owner()) revert NotBuyer();

        uint256 fee = (e.amount * feeBps) / 10000;
        uint256 sellerAmount = e.amount - fee;

        e.status = Status.Released;
        e.releasedAt = uint64(block.timestamp);
        e.fee = fee;

        if (fee > 0) {
            IERC20(e.token).safeTransfer(feeRecipient, fee);
        }
        IERC20(e.token).safeTransfer(e.seller, sellerAmount);

        emit EscrowReleased(escrowId, e.seller, sellerAmount, fee);
    }

    function refundEscrow(bytes32 escrowId) external nonReentrant {
        Escrow storage e = escrows[escrowId];
        if (e.buyer == address(0)) revert EscrowNotFound();
        if (e.status != Status.Active) revert EscrowNotActive();
        if (msg.sender != e.seller && msg.sender != owner()) revert NotBuyerOrSeller();

        e.status = Status.Refunded;
        IERC20(e.token).safeTransfer(e.buyer, e.amount);

        emit EscrowRefunded(escrowId, e.buyer, e.amount);
    }

    function disputeEscrow(bytes32 escrowId) external {
        Escrow storage e = escrows[escrowId];
        if (e.buyer == address(0)) revert EscrowNotFound();
        if (e.status != Status.Active) revert EscrowNotActive();
        if (msg.sender != e.buyer && msg.sender != e.seller) revert NotBuyerOrSeller();

        e.status = Status.Disputed;
        emit EscrowDisputed(escrowId, msg.sender);
    }

    function resolveDispute(bytes32 escrowId, bool releaseToSeller) external onlyOwner {
        Escrow storage e = escrows[escrowId];
        if (e.buyer == address(0)) revert EscrowNotFound();
        if (e.status != Status.Disputed) revert EscrowNotActive();

        if (releaseToSeller) {
            uint256 fee = (e.amount * feeBps) / 10000;
            uint256 sellerAmount = e.amount - fee;
            e.status = Status.Released;
            e.fee = fee;
            if (fee > 0) IERC20(e.token).safeTransfer(feeRecipient, fee);
            IERC20(e.token).safeTransfer(e.seller, sellerAmount);
            emit EscrowReleased(escrowId, e.seller, sellerAmount, fee);
        } else {
            e.status = Status.Refunded;
            IERC20(e.token).safeTransfer(e.buyer, e.amount);
            emit EscrowRefunded(escrowId, e.buyer, e.amount);
        }
    }

    function getEscrow(bytes32 escrowId) external view returns (Escrow memory) {
        return escrows[escrowId];
    }

    function setFee(uint16 _feeBps) external onlyOwner {
        if (_feeBps > 1000) revert FeeTooHigh();
        feeBps = _feeBps;
    }

    function setFeeRecipient(address _feeRecipient) external onlyOwner {
        if (_feeRecipient == address(0)) revert ZeroAddress();
        feeRecipient = _feeRecipient;
    }

    function pause() external onlyOwner { _pause(); }
    function unpause() external onlyOwner { _unpause(); }
}
