// Frontend Environment Configuration - Web3 Enterprise System  
// Pure Web3 authentication with shared configuration system
// Migrated to use shared/config compatibility layers

import { 
  serverEnv, 
  clientEnv, 
  env, 
  urls,
  isServer,
  isDev,
  isProd,
  isStaging
} from '../../../shared/env/schema';

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
  // Enterprise API endpoints
  authenticate: `${env.BACKEND_URL}/api/v1/enterprise/auth/verify`,
  challenge: `${env.BACKEND_URL}/api/v1/enterprise/auth/challenge`,
  permissions: `${env.BACKEND_URL}/api/v1/enterprise/auth/permissions`,
  marketplace: `${env.BACKEND_URL}/api/v1/enterprise/marketplace`,
  billing: `${env.BACKEND_URL}/api/v1/enterprise/billing`,
  analytics: `${env.BACKEND_URL}/api/v1/enterprise/analytics`,
  
  // Health and status
  health: `${env.BACKEND_URL}/api/v1/enterprise/health`,
  status: `${env.BACKEND_URL}/api/v1/enterprise/status`,
  tiers: `${env.BACKEND_URL}/api/v1/enterprise/tiers`,
} as const;

/**
 * Validation and debugging (development only)
 */
if (typeof window !== 'undefined' && isDev) {
  console.log('✅ Web3 Enterprise Frontend Configuration Loaded');
  console.log('🔧 Client Configuration:', {
    backendUrl: config.backendUrl,
    appUrl: config.appUrl,
    adminUrl: config.adminUrl,
    web3Network: config.web3.networkId,
    environment: process.env.NODE_ENV
  });
}

// Export unified schema components for direct access if needed
export { serverEnv, clientEnv, env, urls };

// Export for Web3 enterprise system
export { env as environment };
export { config as clientConfig };
export { enterpriseUrls as apiUrls };
export default config;