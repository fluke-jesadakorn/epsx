/**
 * Contract Addresses and Chain Utilities
 * Manages smart contract addresses across different chains
 */

import bscTestnetDeployment from '@/../contracts/deployments/bsc-testnet.json';
import localhostDeployment from '@/../contracts/deployments/localhost.json';

// Chain IDs
export const CHAIN_IDS = {
  BSC_MAINNET: 56,
  BSC_TESTNET: 97,
  LOCALHOST: 31337,
} as const;

// Explorer URLs
const EXPLORER_URLS = {
  [CHAIN_IDS.BSC_MAINNET]: 'https://bscscan.com',
  [CHAIN_IDS.BSC_TESTNET]: 'https://testnet.bscscan.com',
  [CHAIN_IDS.LOCALHOST]: 'http://localhost:8545',
} as const;

// Export as CHAIN_EXPLORERS for backwards compatibility
export const CHAIN_EXPLORERS = EXPLORER_URLS;

// Contract addresses by chain
const PAYMENT_ESCROW_ADDRESSES = {
  [CHAIN_IDS.BSC_TESTNET]: bscTestnetDeployment.contracts.PaymentEscrow.address,
  [CHAIN_IDS.LOCALHOST]: localhostDeployment.contractAddress,
  [CHAIN_IDS.BSC_MAINNET]: '', // To be deployed
} as const;

// Export as PAYMENT_ESCROW_ADDRESS for backwards compatibility
export const PAYMENT_ESCROW_ADDRESS = PAYMENT_ESCROW_ADDRESSES;

// Token addresses by chain
const TOKEN_ADDRESSES = {
  [CHAIN_IDS.BSC_TESTNET]: {
    USDT: bscTestnetDeployment.enabledTokens.USDT.address,
    USDC: bscTestnetDeployment.enabledTokens.USDC.address,
  },
  [CHAIN_IDS.LOCALHOST]: {
    USDT: localhostDeployment.tokens.USDT,
    USDC: localhostDeployment.tokens.USDC,
  },
  [CHAIN_IDS.BSC_MAINNET]: {
    USDT: '0x55d398326f99059fF775485246999027B3197955', // BSC Mainnet USDT
    USDC: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d', // BSC Mainnet USDC
  },
} as const;

/**
 * Get explorer transaction URL for a given chain
 */
export function getExplorerTxUrl(chainId: number, txHash: string): string {
  const baseUrl = EXPLORER_URLS[chainId as keyof typeof EXPLORER_URLS];
  if (!baseUrl) {
    return `https://bscscan.com/tx/${txHash}`; // Fallback to mainnet
  }
  if (chainId === CHAIN_IDS.LOCALHOST) {
    return `${baseUrl}#${txHash}`; // Local explorer format
  }
  return `${baseUrl}/tx/${txHash}`;
}

/**
 * Get explorer address URL for a given chain
 */
export function getExplorerAddressUrl(chainId: number, address: string): string {
  const baseUrl = EXPLORER_URLS[chainId as keyof typeof EXPLORER_URLS];
  if (!baseUrl) {
    return `https://bscscan.com/address/${address}`; // Fallback to mainnet
  }
  if (chainId === CHAIN_IDS.LOCALHOST) {
    return `${baseUrl}#${address}`; // Local explorer format
  }
  return `${baseUrl}/address/${address}`;
}

/**
 * Get payment escrow contract address for a given chain
 */
export function getPaymentEscrowAddress(chainId: number): string {
  const address = PAYMENT_ESCROW_ADDRESSES[chainId as keyof typeof PAYMENT_ESCROW_ADDRESSES];
  if (!address) {
    throw new Error(`PaymentEscrow not deployed on chain ${chainId}`);
  }
  return address;
}

/**
 * Get payment receiver address (same as PaymentEscrow)
 */
export function getPaymentReceiverAddress(chainId: number): string {
  return getPaymentEscrowAddress(chainId);
}

/**
 * Get token address for a given symbol and chain
 */
export function getTokenAddress(symbol: 'USDT' | 'USDC', chainId: number): string {
  const tokens = TOKEN_ADDRESSES[chainId as keyof typeof TOKEN_ADDRESSES];
  if (!tokens) {
    throw new Error(`No token addresses configured for chain ${chainId}`);
  }
  const address = tokens[symbol];
  if (!address) {
    throw new Error(`Token ${symbol} not configured for chain ${chainId}`);
  }
  return address;
}

/**
 * Check if payment escrow is deployed on a given chain
 */
export function isPaymentEscrowDeployed(chainId: number): boolean {
  const address = PAYMENT_ESCROW_ADDRESSES[chainId as keyof typeof PAYMENT_ESCROW_ADDRESSES];
  return Boolean(address);
}
