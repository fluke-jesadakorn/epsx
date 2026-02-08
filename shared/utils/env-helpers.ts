/**
 * Environment Helpers for EPSX Platform
 * 
 * Used to conditionally show/hide UI features based on deployment environment.
 * Features are hidden in production but code stays for future multi-chain support.
 */

import { logger } from './logger';

// Get blockchain network from environment
const blockchainNetwork = process.env['NEXT_PUBLIC_BLOCKCHAIN_NETWORK'] ?? 'testnet';

/**
 * Check if running on localhost or 127.0.0.1
 */
export const isLocalHost = typeof window !== 'undefined'
    ? window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
    : process.env['NODE_ENV'] === 'development';

/**
 * Check if running in production (mainnet) environment
 * Use this to HIDE dev-only features
 */
export const isProduction = blockchainNetwork === 'mainnet';

/**
 * Check if running in testnet environment
 */
export const isTestnet = blockchainNetwork === 'testnet';

/**
 * Check if running in local development (Anvil) environment
 */
export const isLocal = blockchainNetwork === 'local' || isLocalHost;

/**
 * Check if running in any development environment (local or testnet)
 * Use this to SHOW dev-only features
 */
export const isDev = isLocal || isTestnet;

/**
 * Get the current blockchain network name
 */
export const getNetworkName = (): string => {
    if (isLocal) { return 'Local (Anvil)'; }
    if (isTestnet) { return 'BSC Testnet'; }
    if (isProduction) { return 'BSC Mainnet'; }
    return 'Unknown';
};

/**
 * Get the chain ID for current environment
 */
export const getDefaultChainId = (): number => {
    if (isLocal) { return 31337; }
    if (isTestnet) { return 97; }
    if (isProduction) { return 56; }
    return 97; // Default to testnet
};

/**
 * Debug log helper - only logs in non-production environments
 */
export const devLog = (...args: unknown[]): void => {
    if (isDev) {
        const [message, ...rest] = args;
        if (typeof message === 'string') {
            logger.debug(message, ...rest);
        } else {
            logger.debug('Dev Log:', ...args);
        }
    }
};

/**
 * Debug warn helper - only warns in non-production environments
 */
export const devWarn = (...args: unknown[]): void => {
    if (isDev) {
        const [message, ...rest] = args;
        if (typeof message === 'string') {
            logger.warn(message, ...rest);
        } else {
            logger.warn('Dev Warning:', ...args);
        }
    }
};
