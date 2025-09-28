// Admin Frontend Environment Configuration - Using Unified Schema
// Now uses /shared/env/schema.ts for consistent validation across all services
// Root .env file loaded via next.config.ts

import { 
  serverEnv, 
  clientEnv, 
  env, 
  legacyUrls as urls,
  isServer,
  isDev,
  isProd,
  isStaging
} from '../../../shared/env/schema';

/**
 * Admin Configuration
 * Uses unified schema with admin-specific defaults
 */
export const config = {
  // Core URLs (admin-specific naming for backward compatibility)
  adminUrl: env.ADMIN_URL,
  backendUrl: env.BACKEND_URL,
  frontendUrl: env.APP_URL,  // Called frontendUrl in admin context
  clientId: 'epsx-admin',   // Fixed client ID for Web3 wallet-first authentication
  
  // Environment flags
  isDev,
  isProd,
  isStaging,
  
  // Port for admin frontend (default from unified schema or environment)
  port: process.env.PORT ? parseInt(process.env.PORT) : 3001,
} as const;

/**
 * Admin Auth Configuration
 * Uses unified URL helpers where possible
 */
export const authConfig = {
  appUrl: env.ADMIN_URL,
  apiUrl: env.BACKEND_URL,
  clientId: 'epsx-admin',  // Fixed client ID for Web3 wallet-first authentication
  callbackPath: '/api/auth/callback/epsx-backend',
  
  get callbackUrl() {
    return `${this.appUrl}${this.callbackPath}`;
  },
  
  get authorizationEndpoint() {
    return urls.oauth.authorize;  // Uses unified URL helper
  },
  
  get tokenEndpoint() {
    return urls.oauth.token;      // Uses unified URL helper
  },
  
  get userinfoEndpoint() {
    return urls.oauth.userinfo;   // Uses unified URL helper
  }
} as const;

/**
 * Feature Flags (Legacy)
 * @deprecated Use shared/config/feature-flags.ts through compatibility layer instead
 * Kept for backward compatibility during migration
 */
export const featureFlags = {
  UNIFIED_USER_MANAGEMENT: process.env.NEXT_PUBLIC_ENABLE_UNIFIED_USERS === 'true',
  SERVER_COMPONENTS: process.env.NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS === 'true', 
  NEW_NAVIGATION: process.env.NEXT_PUBLIC_ENABLE_NEW_NAV === 'true',
  DEV_MODE: isDev,
  
  // Legacy rollout percentages (for compatibility)
  rolloutPercentages: {
    UNIFIED_USER_MANAGEMENT: 100,
    SERVER_COMPONENTS: 50,
    NEW_NAVIGATION: 75,
  } as const,
} as const;

/**
 * Server-only configuration for admin frontend
 * Uses unified schema's server-only getters
 */
export const serverConfig = {
  get jwtSecret() {
    return env.WEB3_APP_SECRET; // Uses unified schema's Web3 app secret
  },
  
  get oidcClientSecret() {
    return env.WEB3_APP_SECRET; // Uses Web3 app secret for authentication
  },
  
  get databaseUrl() {
    return env.DATABASE_URL; // Uses unified schema's server-only getter
  },
  
  get redisUrl() {
    return env.REDIS_URL; // Uses unified schema's server-only getter
  }
} as const;

/**
 * Development validation
 */
if (typeof window !== 'undefined' && isDev) {
  console.log('✅ Admin Frontend Environment Configuration Loaded (Unified Schema)');
  console.log('🔧 Admin Configuration:', {
    adminUrl: config.adminUrl,
    backendUrl: config.backendUrl,
    clientId: config.clientId,
    environment: process.env.NODE_ENV
  });
}

// Export unified schema components for direct access if needed
export { serverEnv, clientEnv, env, urls };

// Export for backward compatibility
export { env as environment };
export default config;