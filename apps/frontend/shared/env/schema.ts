// Unified Environment Schema for EPSX Platform
// Single source of truth for all environment variables across services
// Created as part of environment refactor to reduce complexity from 80+ to 15 essential vars

import { z } from 'zod';

// Environment context detection
export const isServer = typeof window === 'undefined';
export const isDev = (typeof process !== 'undefined' && process.env?.NODE_ENV === 'development') || false;
export const isProd = (typeof process !== 'undefined' && process.env?.NODE_ENV === 'production') || false;
export const isStaging = (typeof process !== 'undefined' && (process.env?.NODE_ENV as string) === 'staging') || false;
export const isBuild = (typeof process !== 'undefined' && (process.env?.NEXT_PHASE === 'phase-production-build' || process.env?.CI === 'true')) || false;

// URL defaults based on environment
const getDefaultBackendUrl = () => {
  if (isDev) return 'http://localhost:8080';
  if (isStaging) return 'https://staging-api.epsx.io';
  return undefined; // Force explicit configuration in production
};

const getDefaultFrontendUrl = () => {
  if (isDev) return 'http://localhost:3000';
  if (isStaging) return 'https://staging.epsx.io';
  return undefined; // Force explicit configuration in production
};

const getDefaultAdminUrl = () => {
  if (isDev) return 'http://localhost:3001';
  if (isStaging) return 'https://staging-admin.epsx.io';
  return undefined; // Force explicit configuration in production
};

// OAuth Client ID defaults
const getDefaultFrontendClientId = () => {
  if (isDev) return 'epsx-frontend';
  if (isStaging) return 'epsx-frontend-staging';
  return 'epsx-frontend-prod'; // Sensible default for production
};

const getDefaultAdminClientId = () => {
  if (isDev) return 'epsx-admin';
  if (isStaging) return 'epsx-admin-staging';
  return 'epsx-admin-prod'; // Sensible default for production
};

/**
 * Server-Only Environment Variables (15 total)
 * These variables are only available on the server-side and contain sensitive data
 * NEVER expose these to the client-side
 */
export const serverEnvSchema = z.object({
  // Core Infrastructure (4 variables) - Required for all services
  DATABASE_URL: z.string()
    .url()
    .refine(url => url.startsWith('postgresql://') || url.startsWith('postgres://'), {
      message: 'DATABASE_URL must be a valid PostgreSQL connection string'
    })
    .describe('PostgreSQL connection string for all database operations'),

  BACKEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultBackendUrl() ?? '')
    .refine(url => isBuild || url !== undefined, {
      message: 'BACKEND_URL must be explicitly set in production environment'
    })
    .describe('Backend API URL for internal service communication and OIDC issuer'),

  FRONTEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultFrontendUrl() ?? '')
    .refine(url => isBuild || url !== undefined, {
      message: 'FRONTEND_URL must be explicitly set in production environment'
    })
    .describe('Frontend application URL for CORS and redirect configuration'),

  ADMIN_FRONTEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultAdminUrl() ?? '')
    .refine(url => isBuild || url !== undefined, {
      message: 'ADMIN_FRONTEND_URL must be explicitly set in production environment'
    })
    .describe('Admin frontend URL for CORS and admin-specific redirects'),

  // Web3 Authentication (1 variable) - Critical for Web3 security  
  NEXTAUTH_SECRET: z.string()
    .min(32, 'JWT secret must be at least 32 characters for security')
    .describe('JWT token signing secret for Web3 session management'),


  // Payment (2 variables) - Optional for payment processing
  MUSEPAY_PARTNER_ID: z.string().optional()
    .describe('MusePay partner identifier for payment processing integration'),

  MUSEPAY_PRIVATE_KEY: z.string().optional()
    .describe('MusePay private key for secure payment transaction signing'),

  // Infrastructure (1 variable) - Optional performance optimization
  REDIS_URL: z.string().url().optional()
    .describe('Redis connection URL for caching and session storage'),

  LOG_LEVEL: z.enum(['trace', 'debug', 'info', 'warn', 'error'])
    .default('info')
    .describe('Application logging level for debug and monitoring')
});

/**
 * Client-Safe Environment Variables (NEXT_PUBLIC_* prefix)
 * These variables are exposed to the browser and must not contain sensitive data
 * Available on both server and client-side
 */
export const clientEnvSchema = z.object({
  // Core URLs (3 variables)
  NEXT_PUBLIC_BACKEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultBackendUrl() ?? '')
    .refine(url => {
      // Skip validation during build or in browser (client-side)
      if (isBuild || typeof window !== 'undefined') return true;
      // Only enforce production validation on server-side
      return url !== undefined;
    }, {
      message: 'NEXT_PUBLIC_BACKEND_URL must be explicitly set in production environment'
    })
    .describe('Backend API URL accessible from client-side'),

  NEXT_PUBLIC_APP_URL: z.string()
    .url()
    .optional()
    .default(getDefaultFrontendUrl() ?? '')
    .refine(url => {
      // Skip validation during build or in browser (client-side)
      if (isBuild || typeof window !== 'undefined') return true;
      // Only enforce production validation on server-side
      return url !== undefined;
    }, {
      message: 'NEXT_PUBLIC_APP_URL must be explicitly set in production environment'
    })
    .describe('Frontend application URL for client navigation'),

  NEXT_PUBLIC_ADMIN_URL: z.string()
    .url()
    .optional()
    .default(getDefaultAdminUrl() ?? '')
    .refine(url => {
      // Skip validation during build or in browser (client-side)
      if (isBuild || typeof window !== 'undefined') return true;
      // Only enforce production validation on server-side
      return url !== undefined;
    }, {
      message: 'NEXT_PUBLIC_ADMIN_URL must be explicitly set in production environment'
    })
    .describe('Admin frontend URL for client navigation'),

  // Web3 Configuration
  NEXT_PUBLIC_BLOCKCHAIN_NETWORK: z.enum(['mainnet', 'testnet'])
    .default('testnet')
    .describe('Web3 blockchain network configuration'),

  NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: z.string()
    .default('epsx-web3-frontend')
    .describe('WalletConnect project ID for wallet connections'),

});

/**
 * Unified Environment Configuration
 * Validates and provides type-safe access to environment variables
 */
export type ServerEnv = z.infer<typeof serverEnvSchema>;
export type ClientEnv = z.infer<typeof clientEnvSchema>;

// Lazy parsing - only parse when accessed to ensure environment variables are loaded
let _serverEnv: ServerEnv | null = null;
let _clientEnv: ClientEnv | null = null;

export const serverEnv = new Proxy({} as ServerEnv, {
  get(target, prop) {
    if (!_serverEnv) {
      try {
        _serverEnv = isServer ? serverEnvSchema.parse(process.env) : {} as ServerEnv;
      } catch (error) {
        if (isBuild) {
          // During build, return empty object to prevent build failures
          _serverEnv = {} as ServerEnv;
        } else {
          throw error;
        }
      }
    }
    return (_serverEnv as any)[prop];
  }
});

export const clientEnv = new Proxy({} as ClientEnv, {
  get(target, prop) {
    if (!_clientEnv) {
      try {
        // Check if we're in a browser environment or if process.env is not available
        const envSource = (typeof process !== 'undefined' && process.env) ? process.env : {};
        _clientEnv = clientEnvSchema.parse(envSource);
      } catch (error) {
        if (isBuild || typeof window !== 'undefined') {
          // During build or in browser, provide fallback values to prevent failures
          // In production browsers, Next.js embeds NEXT_PUBLIC_ variables at build time
          _clientEnv = {
            NEXT_PUBLIC_BACKEND_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_BACKEND_URL) || 'https://api.epsx.io',
            NEXT_PUBLIC_APP_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_APP_URL) || 'https://epsx.io',
            NEXT_PUBLIC_ADMIN_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_ADMIN_URL) || 'https://admin.epsx.io',
            NEXT_PUBLIC_BLOCKCHAIN_NETWORK: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_BLOCKCHAIN_NETWORK) || 'testnet',
            NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID) || 'epsx-web3-frontend'
          } as ClientEnv;
        } else {
          throw error;
        }
      }
    }
    return (_clientEnv as any)[prop];
  }
});

/**
 * Environment Getters - Simple, type-safe access to environment variables
 */
export const env = {
  // Always available (client-safe) - using getters to make them lazy
  get BACKEND_URL() {
    return clientEnv.NEXT_PUBLIC_BACKEND_URL;
  },
  get APP_URL() {
    return clientEnv.NEXT_PUBLIC_APP_URL;
  },
  get ADMIN_URL() {
    return clientEnv.NEXT_PUBLIC_ADMIN_URL;
  },
  get BLOCKCHAIN_NETWORK() {
    return clientEnv.NEXT_PUBLIC_BLOCKCHAIN_NETWORK;
  },

  get WALLETCONNECT_PROJECT_ID() {
    return clientEnv.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID;
  },

  // Server-only (throws error if accessed on client)
  get DATABASE_URL() {
    if (!isServer) throw new Error('DATABASE_URL is server-only');
    return serverEnv.DATABASE_URL;
  },

  get JWT_SECRET() {
    if (!isServer) throw new Error('NEXTAUTH_SECRET is server-only');
    return serverEnv.NEXTAUTH_SECRET;
  },

  // Firebase removed - Web3 wallet-first authentication migration completed

  get REDIS_URL() {
    if (!isServer) throw new Error('REDIS_URL is server-only');
    return serverEnv.REDIS_URL;
  }
};

/**
 * Environment URL Helpers - Consistent URL construction
 */
export const urls = {
  get backend() {
    return env.BACKEND_URL;
  },
  get frontend() {
    return env.APP_URL;
  },
  get admin() {
    return env.ADMIN_URL;
  },

  // OIDC endpoints
  oauth: {
    get authorize() {
      return `${env.BACKEND_URL}/oauth/authorize`;
    },
    get token() {
      return `${env.BACKEND_URL}/oauth/token`;
    },
    get userinfo() {
      return `${env.BACKEND_URL}/oauth/userinfo`;
    },
    get jwks() {
      return `${env.BACKEND_URL}/oauth/jwks`;
    }
  },

  // Callback URLs
  callbacks: {
    get frontend() {
      return `${env.APP_URL}/api/auth/callback/epsx-backend`;
    },
    get admin() {
      return `${env.ADMIN_URL}/api/auth/callback/epsx-backend`;
    }
  }
};

/**
 * Environment Validation Summary
 * Logs validation results for debugging - deferred to avoid immediate evaluation
 */
export function logEnvironmentDebugInfo() {
  if (typeof window !== 'undefined' && isDev) {
    console.log('✅ EPSX Web3 Environment Schema Loaded');
    console.log('🔧 Client Environment Variables:', {
      BACKEND_URL: env.BACKEND_URL,
      APP_URL: env.APP_URL,
      ADMIN_URL: env.ADMIN_URL,
      BLOCKCHAIN_NETWORK: env.BLOCKCHAIN_NETWORK,
      WALLETCONNECT_PROJECT_ID: env.WALLETCONNECT_PROJECT_ID
    });
  }
}

// Note: Schemas are already exported above