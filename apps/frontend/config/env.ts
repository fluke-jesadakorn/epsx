// Frontend Environment Configuration - Using Unified Schema
// Now uses /shared/env/schema.ts for consistent validation across all services
// Root .env file loaded via next.config.ts

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
 * Type-safe environment configuration
 * Uses unified schema with proper client/server separation
 */
export const config = {
  // Always available (client-safe URLs)
  backendUrl: env.BACKEND_URL,
  appUrl: env.APP_URL,
  adminUrl: env.ADMIN_URL,
  clientId: env.CLIENT_ID,
  
  // Firebase client configuration
  firebase: {
    apiKey: env.FIREBASE_API_KEY,
    projectId: env.FIREBASE_PROJECT_ID,
    authDomain: env.FIREBASE_AUTH_DOMAIN,
    storageBucket: env.FIREBASE_STORAGE_BUCKET,
    messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
    appId: env.FIREBASE_APP_ID,
    measurementId: env.FIREBASE_MEASUREMENT_ID,
  },
  
  // Environment flags
  isDev,
  isProd,
  isStaging,
} as const;

/**
 * Server-only configuration
 * Uses unified schema's server-only getters that throw errors if accessed on client
 */
export const serverConfig = {
  get backendUrl() {
    return env.BACKEND_URL; // Uses unified schema's server-only access
  },
  
  get siteUrl() {
    return env.APP_URL; // Frontend site URL for redirects
  },
  
  get jwtSecret() {
    return env.JWT_SECRET; // Uses unified schema's server-only getter
  },
  
  get oidcClientSecret() {
    return env.OIDC_CLIENT_SECRET; // Uses unified schema's server-only getter
  },
  
  get databaseUrl() {
    return env.DATABASE_URL; // Uses unified schema's server-only getter
  },
  
  get redisUrl() {
    return env.REDIS_URL; // Uses unified schema's server-only getter
  }
} as const;

/**
 * OAuth/OIDC URLs - Uses unified URL helpers
 */
export const oauthUrls = {
  authorize: urls.oidc.authorize,
  token: urls.oidc.token,
  userinfo: urls.oidc.userinfo,
  jwks: urls.oidc.jwks,
  
  // Callback URLs
  callback: urls.callbacks.frontend,
  adminCallback: urls.callbacks.admin,
} as const;

/**
 * Validation and debugging (development only)
 */
if (typeof window !== 'undefined' && isDev) {
  console.log('✅ Frontend Environment Configuration Loaded (Unified Schema)');
  console.log('🔧 Client Configuration:', {
    backendUrl: config.backendUrl,
    appUrl: config.appUrl,
    adminUrl: config.adminUrl,
    clientId: config.clientId,
    environment: process.env.NODE_ENV
  });
}

// Export unified schema components for direct access if needed
export { serverEnv, clientEnv, env, urls };

// Export for backward compatibility
export { env as environment };
export { config as clientConfig };
export { oauthUrls as authConfig };
export default config;