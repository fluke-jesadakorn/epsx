import { expect } from "chai";
import { ethers } from "hardhat";
import { PaymentEscrow, IERC20 } from "../typechain-types";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("PaymentEscrow", function () {
  let paymentEscrow: PaymentEscrow;
  let mockUSDT: IERC20;
  let mockUSDC: IERC20;
  let owner: SignerWithAddress;
  let user1: SignerWithAddress;
  let user2: SignerWithAddress;
  let recipient: SignerWithAddress;

  // Token decimals (USDT/USDC use 6 decimals)
  const DECIMALS = 6;
  const USDT_AMOUNT = (amount: number) => ethers.parseUnits(amount.toString(), DECIMALS);

  // Plan IDs and prices
  const PLAN_STARTER = 1;
  const PLAN_PROFESSIONAL = 2;
  const PLAN_ENTERPRISE = 3;
  const PRICE_STARTER = USDT_AMOUNT(29);
  const PRICE_PROFESSIONAL = USDT_AMOUNT(59);
  const PRICE_ENTERPRISE = USDT_AMOUNT(99);

  beforeEach(async function () {
    // Get signers
    [owner, user1, user2, recipient] = await ethers.getSigners();

    // Deploy mock ERC20 tokens (representing USDT and USDC)
    const MockERC20 = await ethers.getContractFactory("MockERC20");
    mockUSDT = await MockERC20.deploy("Tether USD", "USDT", DECIMALS);
    mockUSDC = await MockERC20.deploy("USD Coin", "USDC", DECIMALS);

    // Deploy PaymentEscrow contract
    const PaymentEscrow = await ethers.getContractFactory("PaymentEscrow");
    paymentEscrow = await PaymentEscrow.deploy(owner.address);

    // Configure contract
    await paymentEscrow.setPlanPrice(PLAN_STARTER, PRICE_STARTER);
    await paymentEscrow.setPlanPrice(PLAN_PROFESSIONAL, PRICE_PROFESSIONAL);
    await paymentEscrow.setPlanPrice(PLAN_ENTERPRISE, PRICE_ENTERPRISE);

    await paymentEscrow.setTokenEnabled(await mockUSDT.getAddress(), true);
    await paymentEscrow.setTokenEnabled(await mockUSDC.getAddress(), true);

    // Mint tokens to users for testing
    await mockUSDT.mint(user1.address, USDT_AMOUNT(1000));
    await mockUSDC.mint(user1.address, USDT_AMOUNT(1000));
    await mockUSDT.mint(user2.address, USDT_AMOUNT(1000));
  });

  describe("Deployment", function () {
    it("Should set the correct owner", async function () {
      expect(await paymentEscrow.owner()).to.equal(owner.address);
    });

    it("Should start unpaused", async function () {
      expect(await paymentEscrow.paused()).to.equal(false);
    });

    it("Should start with zero total payments", async function () {
      expect(await paymentEscrow.getTotalPayments()).to.equal(0);
    });
  });

  describe("Plan Configuration", function () {
    it("Should allow owner to set plan prices", async function () {
      expect(await paymentEscrow.getPlanPrice(PLAN_STARTER)).to.equal(PRICE_STARTER);
      expect(await paymentEscrow.getPlanPrice(PLAN_PROFESSIONAL)).to.equal(PRICE_PROFESSIONAL);
      expect(await paymentEscrow.getPlanPrice(PLAN_ENTERPRISE)).to.equal(PRICE_ENTERPRISE);
    });

    it("Should emit PlanPriceUpdated event", async function () {
      const newPrice = USDT_AMOUNT(39);
      await expect(paymentEscrow.setPlanPrice(PLAN_STARTER, newPrice))
        .to.emit(paymentEscrow, "PlanPriceUpdated")
        .withArgs(PLAN_STARTER, PRICE_STARTER, newPrice);
    });

    it("Should revert if non-owner tries to set price", async function () {
      await expect(
        paymentEscrow.connect(user1).setPlanPrice(PLAN_STARTER, USDT_AMOUNT(100))
      ).to.be.revertedWithCustomError(paymentEscrow, "OwnableUnauthorizedAccount");
    });

    it("Should revert if price is zero", async function () {
      await expect(paymentEscrow.setPlanPrice(PLAN_STARTER, 0)).to.be.revertedWith(
        "Price must be greater than 0"
      );
    });

    it("Should revert if plan ID is zero", async function () {
      await expect(paymentEscrow.setPlanPrice(0, USDT_AMOUNT(50))).to.be.revertedWith(
        "Invalid plan ID"
      );
    });
  });

  describe("Token Management", function () {
    it("Should allow owner to enable tokens", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      expect(await paymentEscrow.isTokenSupported(usdtAddress)).to.equal(true);
    });

    it("Should allow owner to disable tokens", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await paymentEscrow.setTokenEnabled(usdtAddress, false);
      expect(await paymentEscrow.isTokenSupported(usdtAddress)).to.equal(false);
    });

    it("Should emit TokenStatusUpdated event", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(paymentEscrow.setTokenEnabled(usdtAddress, false))
        .to.emit(paymentEscrow, "TokenStatusUpdated")
        .withArgs(usdtAddress, false);
    });

    it("Should revert if non-owner tries to set token status", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.connect(user1).setTokenEnabled(usdtAddress, false)
      ).to.be.revertedWithCustomError(paymentEscrow, "OwnableUnauthorizedAccount");
    });

    it("Should revert if token address is zero", async function () {
      await expect(
        paymentEscrow.setTokenEnabled(ethers.ZeroAddress, true)
      ).to.be.revertedWith("Invalid token address");
    });
  });

  describe("Payment Processing", function () {
    beforeEach(async function () {
      // Approve contract to spend user's tokens
      const escrowAddress = await paymentEscrow.getAddress();
      const usdtAddress = await mockUSDT.getAddress();
      await mockUSDT.connect(user1).approve(escrowAddress, USDT_AMOUNT(1000));
      await mockUSDC.connect(user1).approve(escrowAddress, USDT_AMOUNT(1000));
    });

    it("Should accept valid payment with USDT", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER)
      )
        .to.emit(paymentEscrow, "PaymentReceived")
        .withArgs(
          user1.address,
          PLAN_STARTER,
          usdtAddress,
          PRICE_STARTER,
          await ethers.provider.getBlockNumber().then(n => n + 1).then(async b => (await ethers.provider.getBlock(b))!.timestamp),
          1
        );
    });

    it("Should accept valid payment with USDC", async function () {
      const usdcAddress = await mockUSDC.getAddress();
      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_PROFESSIONAL, usdcAddress, PRICE_PROFESSIONAL)
      ).to.emit(paymentEscrow, "PaymentReceived");
    });

    it("Should increment total payments counter", async function () {
      const usdtAddress = await mockUSDT.getAddress();

      await paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER);
      expect(await paymentEscrow.getTotalPayments()).to.equal(1);

      await paymentEscrow.connect(user1).payForPlan(PLAN_PROFESSIONAL, usdtAddress, PRICE_PROFESSIONAL);
      expect(await paymentEscrow.getTotalPayments()).to.equal(2);
    });

    it("Should transfer tokens to contract", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const escrowAddress = await paymentEscrow.getAddress();
      const balanceBefore = await mockUSDT.balanceOf(escrowAddress);

      await paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER);

      const balanceAfter = await mockUSDT.balanceOf(escrowAddress);
      expect(balanceAfter - balanceBefore).to.equal(PRICE_STARTER);
    });

    it("Should revert if token is not supported", async function () {
      const unsupportedToken = await (await ethers.getContractFactory("MockERC20"))
        .deploy("Fake Token", "FAKE", 18);

      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, await unsupportedToken.getAddress(), PRICE_STARTER)
      ).to.be.revertedWith("Token not supported");
    });

    it("Should revert if plan ID is invalid", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.connect(user1).payForPlan(999, usdtAddress, PRICE_STARTER)
      ).to.be.revertedWith("Invalid plan ID");
    });

    it("Should revert if amount is incorrect", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const wrongAmount = USDT_AMOUNT(50); // Wrong amount for PLAN_STARTER

      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, wrongAmount)
      ).to.be.revertedWith("Incorrect payment amount");
    });

    it("Should revert if user has insufficient balance", async function () {
      const usdtAddress = await mockUSDT.getAddress();

      // user2 has tokens but not approved
      await expect(
        paymentEscrow.connect(user2).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER)
      ).to.be.reverted;
    });

    it("Should revert if contract is paused", async function () {
      await paymentEscrow.pause();
      const usdtAddress = await mockUSDT.getAddress();

      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER)
      ).to.be.revertedWithCustomError(paymentEscrow, "EnforcedPause");
    });
  });

  describe("Fund Withdrawal", function () {
    beforeEach(async function () {
      // Make some payments first
      const escrowAddress = await paymentEscrow.getAddress();
      const usdtAddress = await mockUSDT.getAddress();
      await mockUSDT.connect(user1).approve(escrowAddress, USDT_AMOUNT(1000));
      await paymentEscrow.connect(user1).payForPlan(PLAN_ENTERPRISE, usdtAddress, PRICE_ENTERPRISE);
    });

    it("Should allow owner to withdraw funds", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const withdrawAmount = USDT_AMOUNT(50);

      await expect(
        paymentEscrow.withdrawFunds(usdtAddress, withdrawAmount, recipient.address)
      )
        .to.emit(paymentEscrow, "FundsWithdrawn")
        .withArgs(usdtAddress, withdrawAmount, recipient.address);

      expect(await mockUSDT.balanceOf(recipient.address)).to.equal(withdrawAmount);
    });

    it("Should revert if non-owner tries to withdraw", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.connect(user1).withdrawFunds(usdtAddress, USDT_AMOUNT(10), recipient.address)
      ).to.be.revertedWithCustomError(paymentEscrow, "OwnableUnauthorizedAccount");
    });

    it("Should revert if withdrawal amount exceeds balance", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const excessiveAmount = USDT_AMOUNT(1000);

      await expect(
        paymentEscrow.withdrawFunds(usdtAddress, excessiveAmount, recipient.address)
      ).to.be.revertedWith("Insufficient contract balance");
    });

    it("Should revert if recipient is zero address", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.withdrawFunds(usdtAddress, USDT_AMOUNT(10), ethers.ZeroAddress)
      ).to.be.revertedWith("Invalid recipient");
    });

    it("Should revert if amount is zero", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      await expect(
        paymentEscrow.withdrawFunds(usdtAddress, 0, recipient.address)
      ).to.be.revertedWith("Amount must be greater than 0");
    });
  });

  describe("Emergency Pause", function () {
    it("Should allow owner to pause contract", async function () {
      await paymentEscrow.pause();
      expect(await paymentEscrow.paused()).to.equal(true);
    });

    it("Should allow owner to unpause contract", async function () {
      await paymentEscrow.pause();
      await paymentEscrow.unpause();
      expect(await paymentEscrow.paused()).to.equal(false);
    });

    it("Should revert if non-owner tries to pause", async function () {
      await expect(
        paymentEscrow.connect(user1).pause()
      ).to.be.revertedWithCustomError(paymentEscrow, "OwnableUnauthorizedAccount");
    });

    it("Should block payments when paused", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const escrowAddress = await paymentEscrow.getAddress();
      await mockUSDT.connect(user1).approve(escrowAddress, PRICE_STARTER);

      await paymentEscrow.pause();

      await expect(
        paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER)
      ).to.be.revertedWithCustomError(paymentEscrow, "EnforcedPause");
    });
  });

  describe("View Functions", function () {
    it("Should return correct plan price", async function () {
      expect(await paymentEscrow.getPlanPrice(PLAN_STARTER)).to.equal(PRICE_STARTER);
    });

    it("Should return correct token support status", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      expect(await paymentEscrow.isTokenSupported(usdtAddress)).to.equal(true);
    });

    it("Should return correct contract token balance", async function () {
      const usdtAddress = await mockUSDT.getAddress();
      const escrowAddress = await paymentEscrow.getAddress();

      // Make a payment
      await mockUSDT.connect(user1).approve(escrowAddress, PRICE_STARTER);
      await paymentEscrow.connect(user1).payForPlan(PLAN_STARTER, usdtAddress, PRICE_STARTER);

      expect(await paymentEscrow.getTokenBalance(usdtAddress)).to.equal(PRICE_STARTER);
    });
  });
});


// Mock ERC20 contract for testing
const MockERC20Source = `
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    uint8 private _decimals;

    constructor(
        string memory name,
        string memory symbol,
        uint8 decimals_
    ) ERC20(name, symbol) {
        _decimals = decimals_;
    }

    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
`;
