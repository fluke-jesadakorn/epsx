/**
 * Smart Contract Addresses for BSC Networks
 *
 * These addresses are populated after deployment:
 * - BSC Testnet (97): For development and testing
 * - BSC Mainnet (56): For production
 *
 * Update these addresses after running deployment scripts in apps/contracts
 */

export const PAYMENT_ESCROW_ADDRESS = {
  // BSC Mainnet (ChainID: 56)
  56: process.env.NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET || '',

  // BSC Testnet (ChainID: 97)
  97: process.env.NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET || '',
} as const;

/**
 * ERC20 Token Addresses on BSC
 */
export const TOKEN_ADDRESSES = {
  USDT: {
    56: '0x55d398326f99059fF775485246999027B3197955', // BSC Mainnet
    97: '0x337610d27c682E347C9cD60BD4b3b107C9d34dDD', // BSC Testnet
  },
  USDC: {
    56: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d', // BSC Mainnet
    97: '0x64544969ed7EBf5f083679233325356EbE738930', // BSC Testnet
  },
} as const;

/**
 * Get payment escrow contract address for current chain
 */
export function getPaymentEscrowAddress(chainId: number): string {
  const address = PAYMENT_ESCROW_ADDRESS[chainId as keyof typeof PAYMENT_ESCROW_ADDRESS];
  if (!address) {
    throw new Error(`Payment escrow contract not deployed on chain ${chainId}`);
  }
  return address;
}

/**
 * Get token address for current chain
 */
export function getTokenAddress(token: 'USDT' | 'USDC', chainId: number): string {
  const address = TOKEN_ADDRESSES[token][chainId as keyof typeof TOKEN_ADDRESSES.USDT];
  if (!address) {
    throw new Error(`${token} not available on chain ${chainId}`);
  }
  return address;
}

/**
 * Check if payment escrow contract is deployed on chain
 */
export function isPaymentEscrowDeployed(chainId: number): boolean {
  return Boolean(PAYMENT_ESCROW_ADDRESS[chainId as keyof typeof PAYMENT_ESCROW_ADDRESS]);
}

/**
 * Chain-specific explorer URLs
 */
export const CHAIN_EXPLORERS = {
  56: {
    name: 'BSCScan',
    url: 'https://bscscan.com',
    tx: (hash: string) => `https://bscscan.com/tx/${hash}`,
    address: (addr: string) => `https://bscscan.com/address/${addr}`,
  },
  97: {
    name: 'BSCScan Testnet',
    url: 'https://testnet.bscscan.com',
    tx: (hash: string) => `https://testnet.bscscan.com/tx/${hash}`,
    address: (addr: string) => `https://testnet.bscscan.com/address/${addr}`,
  },
} as const;

/**
 * Get explorer URL for transaction
 */
export function getExplorerTxUrl(chainId: number, txHash: string): string {
  const explorer = CHAIN_EXPLORERS[chainId as keyof typeof CHAIN_EXPLORERS];
  if (!explorer) {
    return `https://bscscan.com/tx/${txHash}`; // Fallback to mainnet
  }
  return explorer.tx(txHash);
}

/**
 * Get explorer URL for address
 */
export function getExplorerAddressUrl(chainId: number, address: string): string {
  const explorer = CHAIN_EXPLORERS[chainId as keyof typeof CHAIN_EXPLORERS];
  if (!explorer) {
    return `https://bscscan.com/address/${address}`; // Fallback to mainnet
  }
  return explorer.address(address);
}
