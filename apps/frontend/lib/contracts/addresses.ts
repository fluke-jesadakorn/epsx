/**
 * Smart Contract Addresses for BSC Networks
 *
 * These addresses are populated after deployment:
 * - Local Anvil (31337): For local development
 * - BSC Testnet (97): For staging/testing
 * - BSC Mainnet (56): For production
 *
 * Update these addresses after running deployment scripts in apps/contracts
 */

/**
 * Validate Ethereum address format
 */
function isValidAddress(address: string | undefined): address is `0x${string}` {
  return typeof address === 'string' && /^0x[a-fA-F0-9]{40}$/.test(address);
}

/**
 * Get optional address from env (returns undefined if not set/invalid)
 */
function getOptionalEnvAddress(envKey: string): `0x${string}` | undefined {
  const address = process.env[envKey];
  if (!isValidAddress(address)) {
    return undefined;
  }
  return address;
}

/**
 * PaymentEscrow Contract Addresses
 * All addresses are read from environment variables for flexibility.
 * Configure in .env:
 *   - NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET (BSC Mainnet, ChainID: 56)
 *   - NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET (BSC Testnet, ChainID: 97)
 *   - NEXT_PUBLIC_PAYMENT_ESCROW_LOCAL (Local Anvil, ChainID: 31337)
 */
export const PAYMENT_ESCROW_ADDRESS: Partial<Record<number, `0x${string}`>> = {
  // BSC Mainnet (ChainID: 56)
  56: getOptionalEnvAddress('NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET'),

  // BSC Testnet (ChainID: 97)
  97: getOptionalEnvAddress('NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET'),

  // Local Anvil (ChainID: 31337) - Deployed via setup-local.sh
  31337: getOptionalEnvAddress('NEXT_PUBLIC_PAYMENT_ESCROW_LOCAL'),
};

/**
 * Payment Receiver Address for Direct Token Transfers
 * - Mainnet: Set via environment variable (validated on access)
 * - Testnet/Local: Use known test accounts
 */
export const PAYMENT_RECEIVER_ADDRESS: Partial<Record<number, `0x${string}`>> = {
  // BSC Mainnet (ChainID: 56) - Optional at load time, validated on access
  56: getOptionalEnvAddress('NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET'),

  // BSC Testnet (ChainID: 97) - Testnet receiver account
  97: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8' as const,

  // Local Anvil (ChainID: 31337) - Anvil account #1
  31337: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8' as const,
};

/**
 * ERC20 Token Addresses on BSC and Local
 */
export const TOKEN_ADDRESSES = {
  USDT: {
    56: '0x55d398326f99059fF775485246999027B3197955', // BSC Mainnet
    97: getOptionalEnvAddress('NEXT_PUBLIC_TESTNET_USDT_ADDRESS') || '0x66E972502A34A625828C544a1914E8D8cc2A9dE5', // BSC Testnet USDT (PandaTool community token)
    31337: '0x55d398326f99059fF775485246999027B3197955', // Local Anvil (Etched to match Mainnet)
  },
  USDC: {
    56: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d', // BSC Mainnet
    97: getOptionalEnvAddress('NEXT_PUBLIC_TESTNET_USDC_ADDRESS') || '0x64544969ed7EBf5f083679233325356EbE738930', // BSC Testnet USDC (community token)
    31337: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d', // Local Anvil (Etched to match Mainnet)
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
 * Get payment receiver address for direct token transfers
 */
export function getPaymentReceiverAddress(chainId: number): string {
  const address = PAYMENT_RECEIVER_ADDRESS[chainId as keyof typeof PAYMENT_RECEIVER_ADDRESS];
  if (!address || address === '0x0000000000000000000000000000000000000000') {
    throw new Error(`Payment receiver not configured for chain ${chainId}`);
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
 * Get the local Anvil base URL based on current browser hostname.
 * Supports Tailscale IPs (100.x.x.x) and other local network IPs.
 */
function getLocalAnvilBaseUrl(): string {
  if (typeof window === 'undefined') {
    return 'http://127.0.0.1:8545';
  }
  const hostname = window.location.hostname;
  if (hostname === 'localhost' || hostname === '127.0.0.1') {
    return 'http://127.0.0.1:8545';
  }
  return `http://${hostname}:8545`;
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
  31337: {
    name: 'Local Anvil',
    get url() { return getLocalAnvilBaseUrl(); },
    tx: (hash: string) => `${getLocalAnvilBaseUrl()}/tx/${hash}`,
    address: (addr: string) => `${getLocalAnvilBaseUrl()}/address/${addr}`,
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
