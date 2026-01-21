// Frontend Environment Configuration - Web3 Enterprise System  
// Pure Web3 authentication with shared configuration system
// Migrated to use shared/config compatibility layers

import {
  clientEnv,
  env,
  isDev,
  isProd,
  isStaging,
  serverEnv,
  urls
} from '@/shared/env/schema';

/**
 * Type-safe Web3 enterprise configuration
 * Client-safe configuration for Web3 authentication
 */
export const config = {
  // Enterprise API URLs
  backendUrl: env.BACKEND_URL,
  appUrl: env.APP_URL,
  adminUrl: env.ADMIN_URL,

  // Web3 configuration (use shared auth config for full Web3 setup)
  web3: {
    networkId: process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK || 'testnet',
    walletConnectProjectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
  },

  // Web3 blockchain configuration
  blockchainNetwork: env.BLOCKCHAIN_NETWORK,
  walletConnectProjectId: env.WALLETCONNECT_PROJECT_ID,

  // Environment flags
  isDev,
  isProd,
  isStaging,
} as const;

/**
 * Server-only configuration for Web3 enterprise backend
 * Uses unified schema's server-only getters that throw errors if accessed on client
 */
export const serverConfig = {
  get backendUrl() {
    return env.BACKEND_URL;
  },

  get siteUrl() {
    return env.APP_URL;
  },

  get databaseUrl() {
    return env.DATABASE_URL;
  },

  get redisUrl() {
    return env.REDIS_URL;
  }
} as const;

/**
 * Web3 Enterprise API URLs
 */
export const enterpriseUrls = {
  // Web3 Authentication endpoints
  authenticate: `${env.BACKEND_URL}/api/auth/web3/verify`,
  challenge: `${env.BACKEND_URL}/api/auth/web3/challenge`,
  permissions: `${env.BACKEND_URL}/api/permissions/validate`,
  session: `${env.BACKEND_URL}/api/auth/web3/session`,
  logout: `${env.BACKEND_URL}/api/auth/web3/logout`,

  // Enterprise API endpoints (when backend supports them)
  marketplace: `${env.BACKEND_URL}/api/enterprise/marketplace`,
  billing: `${env.BACKEND_URL}/api/enterprise/billing`,
  analytics: `${env.BACKEND_URL}/api/analytics`,

  // Health and status
  health: `${env.BACKEND_URL}/health`,
  status: `${env.BACKEND_URL}/api/enterprise/status`,
  tiers: `${env.BACKEND_URL}/api/plans`,
} as const;

/**
 * Validation and debugging (development only)
 */
// Debug logging removed - use browser devtools for environment inspection

// Export unified schema components for direct access if needed
export { clientEnv, env, serverEnv, urls };

// Export for Web3 enterprise system
export { enterpriseUrls as apiUrls, config as clientConfig, env as environment };
export default config;