/**
 * PaymentEscrow Contract ABI
 *
 * This is the Application Binary Interface for the PaymentEscrow smart contract.
 * It defines how to interact with the contract functions and events.
 *
 * Generated from: apps/contracts/contracts/PaymentEscrow.sol
 */

export const PAYMENT_ESCROW_ABI = [
  {
    type: 'constructor',
    inputs: [
      { name: 'initialOwner', type: 'address', internalType: 'address' }
    ],
    stateMutability: 'nonpayable',
  },
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
    name: 'setPlanPrice',
    inputs: [
      { name: 'planId', type: 'uint256', internalType: 'uint256' },
      { name: 'price', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
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
  {
    type: 'function',
    name: 'getPlanPrice',
    inputs: [
      { name: 'planId', type: 'uint256', internalType: 'uint256' }
    ],
    outputs: [
      { name: '', type: 'uint256', internalType: 'uint256' }
    ],
    stateMutability: 'view',
  },
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
    name: 'owner',
    inputs: [],
    outputs: [
      { name: '', type: 'address', internalType: 'address' }
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
    type: 'event',
    name: 'PaymentReceived',
    inputs: [
      { name: 'user', type: 'address', indexed: true, internalType: 'address' },
      { name: 'planId', type: 'uint256', indexed: true, internalType: 'uint256' },
      { name: 'token', type: 'address', indexed: true, internalType: 'address' },
      { name: 'amount', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'timestamp', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'paymentId', type: 'uint256', indexed: false, internalType: 'uint256' }
    ],
    anonymous: false,
  },
  {
    type: 'event',
    name: 'PlanPriceUpdated',
    inputs: [
      { name: 'planId', type: 'uint256', indexed: true, internalType: 'uint256' },
      { name: 'oldPrice', type: 'uint256', indexed: false, internalType: 'uint256' },
      { name: 'newPrice', type: 'uint256', indexed: false, internalType: 'uint256' }
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
export interface PaymentReceivedEvent {
  user: string;
  planId: bigint;
  token: string;
  amount: bigint;
  timestamp: bigint;
  paymentId: bigint;
}

export interface PlanPriceUpdatedEvent {
  planId: bigint;
  oldPrice: bigint;
  newPrice: bigint;
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
