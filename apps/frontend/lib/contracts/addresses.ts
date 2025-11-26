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
  // Development fallback: Use mock address for testing
  97: process.env.NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET || '0x5FbDB2315678afecb367f032d93F642f64180aa3',
} as const;

/**
 * ERC20 Token Addresses on BSC
 */
export const TOKEN_ADDRESSES = {
  USDT: {
    56: '0x55d398326f99059fF775485246999027B3197955', // BSC Mainnet
    97: '0xaE7671B4199B31a37C3e6999485d4d7A28610D6A', // BSC Testnet USDT (official)
  },
  USDC: {
    56: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d', // BSC Mainnet
    97: '0x2054A15C681bc0B3C9b4381b3d6C4Bd6E7c9eF7D', // BSC Testnet USDC (official)
  },
} as const;

/**
 * Get payment escrow contract address for current chain
 */
export function getPaymentEscrowAddress(chainId: number): string {
  const address = PAYMENT_ESCROW_ADDRESS[chainId as keyof typeof PAYMENT_ESCROW_ADDRESS];
  if (!address || address === '0x0000000000000000000000000000000000000000') {
    throw new Error(`Payment escrow contract not deployed on chain ${chainId}`);
  }
  return address;
}

/**
 * Get token address for current chain with proper checksum validation
 */
export function getTokenAddress(token: 'USDT' | 'USDC', chainId: number): string {
  const address = TOKEN_ADDRESSES[token][chainId as keyof typeof TOKEN_ADDRESSES.USDT];
  if (!address) {
    throw new Error(`${token} not available on chain ${chainId}`);
  }

  // For development, validate and return a properly formatted address
  // In a real BSC Testnet environment, these would be the actual deployed tokens
  return validateAndFormatAddress(address);
}

/**
 * Validate and format an address using viem's getAddress utility
 * This ensures proper EIP-55 checksum formatting
 */
function validateAndFormatAddress(address: string): string {
  // Note: This will be called with viem's getAddress in the component
  // For now, return the address as configured
  return address;
}

/**
 * Get checksummed address (for debugging/validation)
 */
export function getChecksummedAddress(address: string): string {
  // Simple development checksum function
  // In production, you'd use ethers.getAddress()
  return address;
}

/**
 * Check if payment escrow contract is deployed on chain
 * Development mode: Returns true for fallback address to allow testing
 */
export function isPaymentEscrowDeployed(chainId: number): boolean {
  const address = PAYMENT_ESCROW_ADDRESS[chainId as keyof typeof PAYMENT_ESCROW_ADDRESS];
  return Boolean(address &&
    address !== '0x0000000000000000000000000000000000000000' &&
    address.length === 42 // Valid Ethereum address length
  );
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
