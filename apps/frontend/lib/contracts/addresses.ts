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

// Contract addresses by chain (env vars override deployment JSON)
const PAYMENT_ESCROW_ADDRESSES: Record<number, string> = {
  [CHAIN_IDS.BSC_TESTNET]:
    process.env.NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET
    ?? bscTestnetDeployment.contracts.PaymentEscrow.address,
  [CHAIN_IDS.LOCALHOST]:
    process.env.NEXT_PUBLIC_PAYMENT_ESCROW_LOCAL
    ?? localhostDeployment.contractAddress,
  [CHAIN_IDS.BSC_MAINNET]:
    process.env.NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET
    ?? '',
};

// Export as PAYMENT_ESCROW_ADDRESS for backwards compatibility
export const PAYMENT_ESCROW_ADDRESS = PAYMENT_ESCROW_ADDRESSES;

// Payment receiver addresses (where tokens actually go)
// Env var override allows sending directly to a wallet instead of escrow
const PAYMENT_RECEIVER_ADDRESSES: Record<number, string> = {
  [CHAIN_IDS.BSC_TESTNET]:
    process.env.NEXT_PUBLIC_PAYMENT_RECEIVER_TESTNET
    ?? PAYMENT_ESCROW_ADDRESSES[CHAIN_IDS.BSC_TESTNET],
  [CHAIN_IDS.LOCALHOST]:
    process.env.NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL
    ?? PAYMENT_ESCROW_ADDRESSES[CHAIN_IDS.LOCALHOST],
  [CHAIN_IDS.BSC_MAINNET]:
    process.env.NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET
    ?? PAYMENT_ESCROW_ADDRESSES[CHAIN_IDS.BSC_MAINNET],
};

// Token addresses by chain (env vars override defaults)
const TOKEN_ADDRESSES: Record<number, Record<string, string>> = {
  [CHAIN_IDS.BSC_TESTNET]: {
    USDT: process.env.NEXT_PUBLIC_TESTNET_USDT_ADDRESS ?? bscTestnetDeployment.enabledTokens.USDT.address,
    USDC: process.env.NEXT_PUBLIC_TESTNET_USDC_ADDRESS ?? bscTestnetDeployment.enabledTokens.USDC.address,
    DAI: process.env.NEXT_PUBLIC_TESTNET_DAI_ADDRESS ?? '',
  },
  [CHAIN_IDS.LOCALHOST]: {
    USDT: process.env.NEXT_PUBLIC_LOCAL_USDT_ADDRESS ?? localhostDeployment.tokens.USDT,
    USDC: process.env.NEXT_PUBLIC_LOCAL_USDC_ADDRESS ?? localhostDeployment.tokens.USDC,
    DAI: process.env.NEXT_PUBLIC_LOCAL_DAI_ADDRESS ?? '',
  },
  [CHAIN_IDS.BSC_MAINNET]: {
    USDT: process.env.NEXT_PUBLIC_MAINNET_USDT_ADDRESS ?? '0x55d398326f99059fF775485246999027B3197955',
    USDC: process.env.NEXT_PUBLIC_MAINNET_USDC_ADDRESS ?? '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d',
    DAI: process.env.NEXT_PUBLIC_MAINNET_DAI_ADDRESS ?? '0x1AF3F329e8BE154074D8769D1FFa4eE058B1DBc3',
  },
};

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
  const address = PAYMENT_ESCROW_ADDRESSES[chainId];
  if (!address) {
    throw new Error(`PaymentEscrow not deployed on chain ${chainId}`);
  }
  return address;
}

/**
 * Get payment receiver address (wallet or escrow depending on env config)
 */
export function getPaymentReceiverAddress(chainId: number): string {
  const address = PAYMENT_RECEIVER_ADDRESSES[chainId];
  if (!address) {
    throw new Error(`No payment receiver configured for chain ${chainId}`);
  }
  return address;
}

/**
 * Get token address for a given symbol and chain
 */
export function getTokenAddress(symbol: string, chainId: number): string {
  const tokens = TOKEN_ADDRESSES[chainId];
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
  const address = PAYMENT_ESCROW_ADDRESSES[chainId];
  return Boolean(address);
}
