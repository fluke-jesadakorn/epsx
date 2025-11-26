// Unified Environment Schema for EPSX Platform - WEB3-FIRST ARCHITECTURE
// Single source of truth for all environment variables across services
// Phase 4.1: Updated to remove OIDC/Firebase auth vars and add Web3 configuration
// Reduced from 80+ legacy vars to 12 essential Web3-first variables

import { z } from 'zod';

// Environment context detection
export const isServer = typeof window === 'undefined';
export const isDev = (typeof process !== 'undefined' && process.env?.NODE_ENV === 'development') || false;
export const isProd = (typeof process !== 'undefined' && process.env?.NODE_ENV === 'production') || false;
export const isStaging = (typeof process !== 'undefined' && process.env?.NODE_ENV === 'production' && process.env?.DEPLOYMENT_ENV === 'staging') || false;
export const isBuild = (typeof process !== 'undefined' && (process.env?.NEXT_PHASE === 'phase-production-build' || process.env?.CI === 'true')) || false;

// URL defaults based on environment
const getDefaultBackendUrl = () => {
  if (isDev) return 'http://localhost:8080';
  if (isStaging) return 'https://staging-api.epsx.io';
  return 'https://api.epsx.io'; // Production default - api.epsx.io maps to backend service
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

// Web3 Configuration Defaults
const getDefaultBlockchainNetwork = () => {
  if (isDev) return 'testnet';
  if (isStaging) return 'testnet';
  return 'mainnet'; // Production uses BSC mainnet
};

const getDefaultWalletConnectProjectId = () => {
  if (isDev) return 'epsx-web3-dev';
  if (isStaging) return 'epsx-web3-staging';
  return 'epsx-web3-prod'; // Production WalletConnect project
};

/**
 * Server-Only Environment Variables (8 total) - WEB3-FIRST ARCHITECTURE
 * These variables are only available on the server-side and contain sensitive data
 * NEVER expose these to the client-side
 * 
 * ⚠️  PHASE 4.1 MIGRATION: Removed OIDC/Firebase auth variables
 * ✅ Added Web3-specific configuration variables
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
    .default(getDefaultBackendUrl())
    .refine(url => isBuild || url !== undefined, { 
      message: 'BACKEND_URL must be explicitly set in production environment' 
    })
    .describe('Backend API URL for internal service communication and OIDC issuer'),
    
  FRONTEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultFrontendUrl())
    .refine(url => isBuild || url !== undefined, { 
      message: 'FRONTEND_URL must be explicitly set in production environment' 
    })
    .describe('Frontend application URL for CORS and redirect configuration'),
    
  ADMIN_FRONTEND_URL: z.string()
    .url()
    .optional()
    .default(getDefaultAdminUrl())
    .refine(url => isBuild || url !== undefined, { 
      message: 'ADMIN_FRONTEND_URL must be explicitly set in production environment' 
    })
    .describe('Admin frontend URL for CORS and admin-specific redirects'),

  // Web3 Authentication (2 variables) - Critical for Web3 security
  WEB3_APP_SECRET: z.string()
    .min(32, 'Web3 app secret must be at least 32 characters for cryptographic security')
    .optional()
    .default('web3-default-secret-for-development-only-change-in-production')
    .describe('Web3 application secret for wallet signature validation and session management'),
    
  WALLET_SIGNATURE_SECRET: z.string()
    .min(32, 'Wallet signature secret must be at least 32 characters for security')
    .optional()
    .default('wallet-signature-secret-development-change-in-production')
    .describe('Secret key for validating and signing wallet-based authentication tokens'),


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
    .default(getDefaultBackendUrl())
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
    .default(getDefaultFrontendUrl())
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
    .default(getDefaultAdminUrl())
    .refine(url => {
      // Skip validation during build or in browser (client-side)
      if (isBuild || typeof window !== 'undefined') return true;
      // Only enforce production validation on server-side
      return url !== undefined;
    }, { 
      message: 'NEXT_PUBLIC_ADMIN_URL must be explicitly set in production environment' 
    })
    .describe('Admin frontend URL for client navigation'),

  // Web3 Configuration (3 variables)
  NEXT_PUBLIC_BLOCKCHAIN_NETWORK: z.enum(['mainnet', 'testnet'])
    .default(getDefaultBlockchainNetwork())
    .describe('Blockchain network: mainnet (BSC 56) or testnet (BSC 97)'),
    
  NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: z.string()
    .default(getDefaultWalletConnectProjectId())
    .describe('WalletConnect project ID for Web3 wallet connections'),
    
  NEXT_PUBLIC_CHAIN_ID: z.string()
    .optional()
    .default('97') // BSC Testnet default for development
    .describe('Blockchain chain ID - automatically determined by network setting'),

  // Payment Configuration (2 variables) - Company wallet addresses for receiving payments
  NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS: z.string()
    .optional()
    .default('0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7')
    .describe('Company wallet address for mainnet payments (BSC mainnet)'),

  NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS: z.string()
    .optional()
    .default('0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7')
    .describe('Company wallet address for testnet payments (BSC testnet)'),

  // Firebase Analytics Configuration (4 variables) - Minimal config for frontend analytics only
  NEXT_PUBLIC_FIREBASE_API_KEY: z.string().optional()
    .describe('Firebase API key for client-side analytics (frontend only)'),
    
  NEXT_PUBLIC_FIREBASE_PROJECT_ID: z.string().optional()
    .describe('Firebase project ID for client-side analytics (frontend only)'),
    
  NEXT_PUBLIC_FIREBASE_APP_ID: z.string().optional()
    .describe('Firebase app ID for client-side analytics (frontend only)'),
    
  NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID: z.string().optional()
    .describe('Firebase measurement ID for analytics tracking (frontend only)')
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
            NEXT_PUBLIC_BACKEND_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_BACKEND_URL) || getDefaultBackendUrl(),
            NEXT_PUBLIC_APP_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_APP_URL) || getDefaultFrontendUrl(),
            NEXT_PUBLIC_ADMIN_URL: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_ADMIN_URL) || getDefaultAdminUrl(),
            NEXT_PUBLIC_BLOCKCHAIN_NETWORK: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_BLOCKCHAIN_NETWORK) || getDefaultBlockchainNetwork(),
            NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID) || getDefaultWalletConnectProjectId(),
            NEXT_PUBLIC_CHAIN_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_CHAIN_ID) || '97',
            // Payment addresses
            NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS) || '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7',
            NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS) || '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7',
            // Firebase Analytics fallbacks - minimal config (frontend only)
            NEXT_PUBLIC_FIREBASE_API_KEY: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_FIREBASE_API_KEY) || undefined,
            NEXT_PUBLIC_FIREBASE_PROJECT_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_FIREBASE_PROJECT_ID) || undefined,
            NEXT_PUBLIC_FIREBASE_APP_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_FIREBASE_APP_ID) || undefined,
            NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID: (typeof process !== 'undefined' && process.env?.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID) || undefined
          } as ClientEnv;
        } else {
          // In development, log the error and provide safe fallbacks
          console.warn('Environment validation failed, using fallbacks:', error);
          _clientEnv = {
            NEXT_PUBLIC_BACKEND_URL: getDefaultBackendUrl() || 'http://localhost:8080',
            NEXT_PUBLIC_APP_URL: getDefaultFrontendUrl() || 'http://localhost:3000',
            NEXT_PUBLIC_ADMIN_URL: getDefaultAdminUrl() || 'http://localhost:3001',
            NEXT_PUBLIC_BLOCKCHAIN_NETWORK: getDefaultBlockchainNetwork(),
            NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: getDefaultWalletConnectProjectId(),
            NEXT_PUBLIC_CHAIN_ID: '97', // BSC Testnet default
            NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS: '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7',
            NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS: '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7'
          } as ClientEnv;
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
  get CHAIN_ID() {
    return clientEnv.NEXT_PUBLIC_CHAIN_ID;
  },

  // Payment Configuration - Company wallet addresses
  get PAYMENT_MAINNET_ADDRESS() {
    return clientEnv.NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS;
  },
  get PAYMENT_TESTNET_ADDRESS() {
    return clientEnv.NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS;
  },

  // Firebase Analytics Configuration (minimal - frontend only)
  get FIREBASE_API_KEY() {
    return clientEnv.NEXT_PUBLIC_FIREBASE_API_KEY;
  },
  get FIREBASE_PROJECT_ID() {
    return clientEnv.NEXT_PUBLIC_FIREBASE_PROJECT_ID;
  },
  get FIREBASE_APP_ID() {
    return clientEnv.NEXT_PUBLIC_FIREBASE_APP_ID;
  },
  get FIREBASE_MEASUREMENT_ID() {
    return clientEnv.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID;
  },
  
  // Server-only (returns undefined if accessed on client)
  get DATABASE_URL() {
    if (!isServer) {
      if (process.env.NODE_ENV === 'development') {
        console.warn('DATABASE_URL is server-only - returning undefined for client');
      }
      return undefined;
    }
    return serverEnv.DATABASE_URL;
  },
  
  get WEB3_APP_SECRET() {
    if (!isServer) {
      if (process.env.NODE_ENV === 'development') {
        console.warn('WEB3_APP_SECRET is server-only - returning undefined for client');
      }
      return undefined;
    }
    return serverEnv.WEB3_APP_SECRET;
  },
  
  get WALLET_SIGNATURE_SECRET() {
    if (!isServer) {
      if (process.env.NODE_ENV === 'development') {
        console.warn('WALLET_SIGNATURE_SECRET is server-only - returning undefined for client');
      }
      return undefined;
    }
    return serverEnv.WALLET_SIGNATURE_SECRET;
  },
  
  
  get REDIS_URL() {
    if (!isServer) {
      if (process.env.NODE_ENV === 'development') {
        console.warn('REDIS_URL is server-only - returning undefined for client');
      }
      return undefined;
    }
    return serverEnv.REDIS_URL;
  }
};

/**
 * Environment URL Helpers - Consistent URL construction
 * 
 * Re-export centralized URL resolver for backward compatibility
 * while maintaining the same API surface
 */
export { urls, getBackendUrl, getFrontendUrl, getAdminUrl, oidcUrls, callbackUrls, apiUrls } from '../utils/url-resolver';

/**
 * Web3 URL helpers for wallet authentication
 */
export const web3Urls = {
  get backend() {
    return env.BACKEND_URL;
  },
  get frontend() {
    return env.APP_URL;
  },
  get admin() {
    return env.ADMIN_URL;
  },
  
  // Web3 authentication endpoints
  auth: {
    get challenge() {
      return `${env.BACKEND_URL}/api/v1/auth/web3/challenge`;
    },
    get verify() {
      return `${env.BACKEND_URL}/api/v1/auth/web3/verify`;
    },
    get permissions() {
      return `${env.BACKEND_URL}/api/v1/auth/web3/permissions`;
    },
    get logout() {
      return `${env.BACKEND_URL}/api/v1/auth/web3/logout`;
    }
  },
  
  // Wallet authentication callbacks
  callbacks: {
    get frontend() {
      return `${env.APP_URL}/api/v1/auth/web3/verify`;
    },
    get admin() {
      return `${env.ADMIN_URL}/api/v1/auth/web3/verify`;
    }
  }
};

/**
 * Environment Validation Summary
 * Logs validation results for debugging - deferred to avoid immediate evaluation
 */
export function logEnvironmentDebugInfo() {
  if (typeof window !== 'undefined' && isDev) {
    console.log('✅ EPSX Environment Schema Loaded (Web3-First)');
    console.log('🔧 Client Environment Variables:', {
      BACKEND_URL: env.BACKEND_URL,
      APP_URL: env.APP_URL,
      ADMIN_URL: env.ADMIN_URL,
      BLOCKCHAIN_NETWORK: env.BLOCKCHAIN_NETWORK,
      WALLETCONNECT_PROJECT_ID: env.WALLETCONNECT_PROJECT_ID,
      CHAIN_ID: env.CHAIN_ID
    });
  }
}

// Note: Schemas are already exported above