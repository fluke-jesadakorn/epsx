/**
 * PaymentEscrow Contract ABI and Constants
 */

import deploymentAbi from '@/../contracts/deployments/PaymentEscrow.json';

// Parse the ABI from the deployment JSON
export const PAYMENT_ESCROW_ABI = JSON.parse(deploymentAbi.abi) as const;

// Context types for dynamic payments
export enum ContextType {
  PLAN = 0,
  GROUP = 1,
  PRODUCT = 2,
  CAMPAIGN = 3,
  CUSTOM = 4,
}

// Zero link hash constant (empty bytes32 for direct payments without link)
export const ZERO_LINK_HASH = '0x0000000000000000000000000000000000000000000000000000000000000000' as const;

// Minimum payment amount (to prevent dust attacks) - typically 1 USDT/USDC (6 decimals)
export const MIN_PAYMENT_AMOUNT = 1_000_000n; // 1 USDT/USDC in wei (6 decimals)
