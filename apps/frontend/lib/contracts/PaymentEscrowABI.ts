/**
 * PaymentEscrow Contract ABI V2
 *
 * This is the Application Binary Interface for the PaymentEscrow V2 smart contract.
 * It defines how to interact with the contract functions and events.
 *
 * Generated from: apps/contracts/contracts/PaymentEscrow.sol
 * Updated: 2025-12-29 - Complete V2 rewrite with context-based payments
 * 
 * Context Types:
 * - 0: PLAN (subscription plan payment)
 * - 1: GROUP (permission group payment)
 * - 2: PRODUCT (one-time product purchase)
 * - 3: CAMPAIGN (promotional campaign payment)
 * - 4: CUSTOM (custom payment link)
 */

// Context type enum values for TypeScript
export enum ContextType {
  PLAN = 0,
  GROUP = 1,
  PRODUCT = 2,
  CAMPAIGN = 3,
  CUSTOM = 4,
}

export const CONTEXT_TYPE_NAMES: Record<ContextType, string> = {
  [ContextType.PLAN]: 'PLAN',
  [ContextType.GROUP]: 'GROUP',
  [ContextType.PRODUCT]: 'PRODUCT',
  [ContextType.CAMPAIGN]: 'CAMPAIGN',
  [ContextType.CUSTOM]: 'CUSTOM',
};

export const PAYMENT_ESCROW_ABI = [
  {
    type: 'constructor',
    inputs: [
      { name: 'initialAdmin', type: 'address', internalType: 'address' }
    ],
    stateMutability: 'nonpayable',
  },
  // ============ Primary Payment Functions ============
  {
    type: 'function',
    name: 'payWithContext',
    inputs: [
      { name: 'contextType', type: 'uint8', internalType: 'enum PaymentEscrow.ContextType' },
      { name: 'contextId', type: 'uint256', internalType: 'uint256' },
      { name: 'linkHash', type: 'bytes32', internalType: 'bytes32' },
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'payWithContextDisplay',
    inputs: [
      { name: 'contextType', type: 'uint8', internalType: 'enum PaymentEscrow.ContextType' },
      { name: 'contextId', type: 'uint256', internalType: 'uint256' },
      { name: 'linkHash', type: 'bytes32', internalType: 'bytes32' },
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'payable',
  },
  // ============ Convenience Payment Functions ============
  {
    type: 'function',
    name: 'payForPlan',
    inputs: [
      { name: 'planId', type: 'uint256', internalType: 'uint256' },
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'payForGroup',
    inputs: [
      { name: 'planId', type: 'uint256', internalType: 'uint256' },
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'payViaLink',
    inputs: [
      { name: 'contextType', type: 'uint8', internalType: 'enum PaymentEscrow.ContextType' },
      { name: 'contextId', type: 'uint256', internalType: 'uint256' },
      { name: 'linkHash', type: 'bytes32', internalType: 'bytes32' },
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  // ============ Admin Functions ============
  {
    type: 'function',
    name: 'setTokenEnabled',
    inputs: [
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'enabled', type: 'bool', internalType: 'bool' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'withdrawFunds',
    inputs: [
      { name: 'token', type: 'address', internalType: 'address' },
      { name: 'amount', type: 'uint256', internalType: 'uint256' },
      { name: 'recipient', type: 'address', internalType: 'address' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'withdrawNative',
    inputs: [
      { name: 'amount', type: 'uint256', internalType: 'uint256' },
      { name: 'recipient', type: 'address', internalType: 'address payable' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'pause',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'unpause',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  // ============ View Functions ============
  {
    type: 'function',
    name: 'isTokenSupported',
    inputs: [
      { name: 'token', type: 'address', internalType: 'address' }
    ],
    outputs: [
      { name: '', type: 'bool', internalType: 'bool' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getTokenBalance',
    inputs: [
      { name: 'token', type: 'address', internalType: 'address' }
    ],
    outputs: [
      { name: '', type: 'uint256', internalType: 'uint256' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getTotalPayments',
    inputs: [],
    outputs: [
      { name: '', type: 'uint256', internalType: 'uint256' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getContextTypeName',
    inputs: [
      { name: 'contextType', type: 'uint8', internalType: 'enum PaymentEscrow.ContextType' }
    ],
    outputs: [
      { name: '', type: 'string', internalType: 'string' }
    ],
    stateMutability: 'pure',
  },
  {
    type: 'function',
    name: 'hasRole',
    inputs: [
      { name: 'role', type: 'bytes32', internalType: 'bytes32' },
      { name: 'account', type: 'address', internalType: 'address' }
    ],
    outputs: [
      { name: '', type: 'bool', internalType: 'bool' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'supportedTokens',
    inputs: [
      { name: '', type: 'address', internalType: 'address' }
    ],
    outputs: [
      { name: '', type: 'bool', internalType: 'bool' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'totalPayments',
    inputs: [],
    outputs: [
      { name: '', type: 'uint256', internalType: 'uint256' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'paused',
    inputs: [],
    outputs: [
      { name: '', type: 'bool', internalType: 'bool' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'MANAGER_ROLE',
    inputs: [],
    outputs: [
      { name: '', type: 'bytes32', internalType: 'bytes32' }
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'DEFAULT_ADMIN_ROLE',
    inputs: [],
    outputs: [
      { name: '', type: 'bytes32', internalType: 'bytes32' }
    ],
    stateMutability: 'view',
  },
  // ============ Events ============
  {
    type: 'event',
    name: 'PaymentWithContext',
    inputs: [
      { name: 'user', type: 'address', indexed: true, internalType: 'address' },
      { name: 'contextType', type: 'uint8', indexed: true, internalType: 'enum PaymentEscrow.ContextType' },
      { name: 'contextId', type: 'uint256', indexed: true, internalType: 'uint256' },
      { name: 'token', type: 'address', indexed: false, internalType: 'address' },
      { name: 'amount', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'timestamp', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'paymentId', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'linkHash', type: 'bytes32', indexed: false, internalType: 'bytes32' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'TokenStatusUpdated',
    inputs: [
      { name: 'token', type: 'address', indexed: true, internalType: 'address' },
      { name: 'enabled', type: 'bool', indexed: false, internalType: 'bool' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'FundsWithdrawn',
    inputs: [
      { name: 'token', type: 'address', indexed: true, internalType: 'address' },
      { name: 'amount', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'recipient', type: 'address', indexed: true, internalType: 'address' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'NativeFundsWithdrawn',
    inputs: [
      { name: 'amount', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'recipient', type: 'address', indexed: true, internalType: 'address' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'Paused',
    inputs: [
      { name: 'account', type: 'address', indexed: false, internalType: 'address' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'Unpaused',
    inputs: [
      { name: 'account', type: 'address', indexed: false, internalType: 'address' }
    ],
    anonymous: false,
  },
] as const;

/**
 * Typed event interfaces for better type safety
 */
export interface PaymentWithContextEvent {
  user: string;
  contextType: ContextType;
  contextId: bigint;
  token: string;
  amount: bigint;
  timestamp: bigint;
  paymentId: bigint;
  linkHash: string;
}

export interface TokenStatusUpdatedEvent {
  token: string;
  enabled: boolean;
}

export interface FundsWithdrawnEvent {
  token: string;
  amount: bigint;
  recipient: string;
}

export interface NativeFundsWithdrawnEvent {
  amount: bigint;
  recipient: string;
}

/**
 * Helper to convert context type string to enum
 */
export function parseContextType(type: string): ContextType {
  const upperType = type.toUpperCase();
  switch (upperType) {
    case 'PLAN': return ContextType.PLAN;
    case 'GROUP': return ContextType.GROUP;
    case 'PRODUCT': return ContextType.PRODUCT;
    case 'CAMPAIGN': return ContextType.CAMPAIGN;
    case 'CUSTOM': return ContextType.CUSTOM;
    default: throw new Error(`Unknown context type: ${type}`);
  }
}

/**
 * Helper to compute link hash from slug
 */
export function computeLinkHash(slug: string): `0x${string}` {
  // Use keccak256 hash of the slug
  // Note: This should match the backend computation
  const encoder = new TextEncoder();
  const data = encoder.encode(slug);
  // In production, use viem's keccak256 or ethers.js
  // For now, return a placeholder that should be replaced with actual implementation
  return `0x${Array.from(data).map(b => b.toString(16).padStart(2, '0')).join('').padEnd(64, '0')}`;
}

/**
 * Zero hash constant for payments without link verification
 */
export const ZERO_LINK_HASH = '0x0000000000000000000000000000000000000000000000000000000000000000' as const;