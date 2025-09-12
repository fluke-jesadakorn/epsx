// Environment configuration for frontend
// All variables read directly from process.env - no shared dependencies
// Comprehensive single source of truth with proper client/server separation

import { z } from 'zod';

// Distinguish server vs client to avoid parsing server-only vars in the browser
const isServer = typeof window === 'undefined';

// Helpers for dynamic defaults
function getFrontendUrl(): string {
  if (process.env.NEXT_PUBLIC_APP_URL) return process.env.NEXT_PUBLIC_APP_URL;
  if (process.env.NODE_ENV === 'production') return 'https://epsx.io';
  return 'http://localhost:3000';
}

function getBackendUrl(): string {
  const url = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL;
  if (url) return url;
  return process.env.NODE_ENV === 'development' ? 'http://localhost:8080' : 'https://api.epsx.io';
}

function getAdminUrl(): string {
  if (process.env.NEXT_PUBLIC_ADMIN_URL) return process.env.NEXT_PUBLIC_ADMIN_URL;
  return process.env.NODE_ENV === 'development' ? 'http://localhost:3001' : 'https://admin.epsx.io';
}

// Server-only schema (evaluated only on the server)
const serverEnvSchema = z.object({
  // Core App Configuration
  NODE_ENV: z.enum(['development', 'staging', 'production']).default('development'),
  PORT: z.string().transform(Number).default(3000),
  APP_URL: z.string().default(getFrontendUrl()),
  SITE_URL: z.string().optional(),

  // Server-Only API URLs (never exposed to client)
  BACKEND_URL: z.string().default(getBackendUrl()),

  // Client-Safe API URLs (NEXT_PUBLIC_ - exposed to browser)
  NEXT_PUBLIC_BACKEND_URL: z.string().default(getBackendUrl()),
  NEXT_PUBLIC_API_URL: z.string().optional(),
  NEXT_PUBLIC_APP_URL: z.string().default(getFrontendUrl()),
  NEXT_PUBLIC_ADMIN_URL: z.string().default(getAdminUrl()),

  // Server-Only Authentication Secrets (never exposed)
  NEXTAUTH_SECRET: z.string().default('dev-secret-key-32-chars-minimum'),
  OIDC_CLIENT_ID: z.string().default('epsx-frontend'),
  OIDC_CLIENT_SECRET: z.string().default('dev-client-secret'),

  // Client-Safe Authentication (NEXT_PUBLIC_ - exposed to browser)
  NEXT_PUBLIC_OAUTH_CLIENT_ID: z.string().optional(),

  // Server-Only Payment Configuration (secrets protected)
  MUSEPAY_PARTNER_ID: z.string().optional(),
  MUSEPAY_PRIVATE_KEY: z.string().optional(),
  MUSEPAY_PUBLIC_KEY: z.string().optional(),
  MUSEPAY_API_URL: z.string().url().optional(),
  MUSEPAY_NOTIFY_URL: z.string().url().optional(),

  // Server-Only Firebase Configuration (service account secrets)
  FIREBASE_TYPE: z.string().optional(),
  FIREBASE_PROJECT_ID: z.string().optional(),
  FIREBASE_PRIVATE_KEY_ID: z.string().optional(),
  FIREBASE_PRIVATE_KEY: z.string().optional(),
  FIREBASE_CLIENT_EMAIL: z.string().optional(),
  FIREBASE_CLIENT_ID: z.string().optional(),
  FIREBASE_AUTH_URI: z.string().url().optional(),
  FIREBASE_TOKEN_URI: z.string().url().optional(),
  FIREBASE_AUTH_PROVIDER_CERT_URL: z.string().url().optional(),
  FIREBASE_CLIENT_CERT_URL: z.string().url().optional(),
  FIREBASE_UNIVERSE_DOMAIN: z.string().optional(),

  // Server-Only Security & Infrastructure
  COOKIE_ENCRYPTION_KEY: z.string().optional(),
  ADMIN_FRONTEND_URL: z.string().optional(),
  FRONTEND_URL: z.string().optional(),
  
  // Server-Only Feature Flag Management
  FEATURE_FLAGS_ENDPOINT: z.string().url().optional(),
  FEATURE_FLAGS_API_KEY: z.string().optional(),

  // Server-Only Monitoring & Performance
  PERFORMANCE_MONITORING: z.string().optional(),

  // Client-Safe Firebase Configuration (NEXT_PUBLIC_ - exposed to browser)
  NEXT_PUBLIC_FIREBASE_API_KEY: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_PROJECT_ID: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_APP_ID: z.string().optional(),

  // Client-Safe Feature Flags (NEXT_PUBLIC_ - exposed to browser)
  NEXT_PUBLIC_ENABLE_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_NEW_NAV: z.string().optional(),
  NEXT_PUBLIC_ENABLE_BUNDLE_OPT: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_NEW_NAV: z.string().optional(),

  // Client-Safe Build Configuration
  NEXT_PUBLIC_BUILD_MODE: z.string().optional(),
});

// Client-only schema (evaluated in the browser). Only expose NEXT_PUBLIC_* vars.
const clientEnvSchema = z.object({
  NODE_ENV: z.enum(['development', 'staging', 'production']).default('development'),
  NEXT_PUBLIC_BACKEND_URL: z.string().default(getBackendUrl()),
  NEXT_PUBLIC_API_URL: z.string().optional(),
  NEXT_PUBLIC_APP_URL: z.string().default(getFrontendUrl()),
  NEXT_PUBLIC_ADMIN_URL: z.string().default(getAdminUrl()),
  NEXT_PUBLIC_OAUTH_CLIENT_ID: z.string().optional(),

  NEXT_PUBLIC_FIREBASE_API_KEY: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_PROJECT_ID: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID: z.string().optional(),
  NEXT_PUBLIC_FIREBASE_APP_ID: z.string().optional(),

  NEXT_PUBLIC_ENABLE_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_NEW_NAV: z.string().optional(),
  NEXT_PUBLIC_ENABLE_BUNDLE_OPT: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_NEW_NAV: z.string().optional(),

  NEXT_PUBLIC_BUILD_MODE: z.string().optional(),
});

export const env = (isServer ? serverEnvSchema : clientEnvSchema).parse(process.env as any);

// Server-Only Configuration (secrets protected, never exposed to client)
export const serverConfig = {
  // API endpoints for server-side communication
  backendUrl: env.BACKEND_URL,
  apiUrl: env.BACKEND_URL,
  
  // App URLs for server operations
  siteUrl: env.SITE_URL || env.APP_URL,
  adminUrl: env.ADMIN_FRONTEND_URL || (
    process.env.NODE_ENV === 'production' 
      ? 'https://admin.epsx.io'
      : 'http://localhost:3001'
  ),
  frontendUrl: env.FRONTEND_URL || (
    process.env.NODE_ENV === 'production'
      ? 'https://epsx.io'
      : 'http://localhost:3000'
  ),
  appUrl: env.APP_URL,
  
  // Authentication secrets
  auth: {
    secret: env.NEXTAUTH_SECRET,
    oidcClientId: env.OIDC_CLIENT_ID,
    oidcSecret: env.OIDC_CLIENT_SECRET,
    jwtSecret: env.NEXTAUTH_SECRET,
  },
  
  // Payment configuration (secrets protected)
  payment: {
    partnerId: env.MUSEPAY_PARTNER_ID || '',
    privateKey: env.MUSEPAY_PRIVATE_KEY || '',
    publicKey: env.MUSEPAY_PUBLIC_KEY || '',
    apiUrl: env.MUSEPAY_API_URL || '',
    notifyUrl: env.MUSEPAY_NOTIFY_URL || '',
  },
  
  // Firebase service account (secrets protected)
  firebase: {
    type: env.FIREBASE_TYPE,
    projectId: env.FIREBASE_PROJECT_ID,
    privateKeyId: env.FIREBASE_PRIVATE_KEY_ID,
    privateKey: env.FIREBASE_PRIVATE_KEY,
    clientEmail: env.FIREBASE_CLIENT_EMAIL,
    clientId: env.FIREBASE_CLIENT_ID,
    authUri: env.FIREBASE_AUTH_URI,
    tokenUri: env.FIREBASE_TOKEN_URI,
    authProviderCertUrl: env.FIREBASE_AUTH_PROVIDER_CERT_URL,
    clientCertUrl: env.FIREBASE_CLIENT_CERT_URL,
    universeDomain: env.FIREBASE_UNIVERSE_DOMAIN,
  },
  
  // Security configuration
  security: {
    cookieEncryptionKey: env.COOKIE_ENCRYPTION_KEY,
  },
  
  // Feature flag management
  featureFlags: {
    endpoint: env.FEATURE_FLAGS_ENDPOINT,
    apiKey: env.FEATURE_FLAGS_API_KEY,
  },
  
  // Monitoring
  monitoring: {
    enabled: env.PERFORMANCE_MONITORING === 'true',
  }
} as const;

// Client-Safe Configuration (exposed to browser via NEXT_PUBLIC_)
export const clientConfig = {
  // API endpoints accessible from client
  backendUrl: env.NEXT_PUBLIC_BACKEND_URL,
  apiUrl: env.NEXT_PUBLIC_BACKEND_URL,
  
  // App URLs for client navigation
  appUrl: env.NEXT_PUBLIC_APP_URL || (typeof window !== 'undefined' ? window.location.origin : ''),
  adminUrl: env.NEXT_PUBLIC_ADMIN_URL,
  
  // Public authentication configuration
  auth: {
    clientId: env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend',
  },
  
  // Firebase client configuration
  firebase: {
    apiKey: env.NEXT_PUBLIC_FIREBASE_API_KEY,
    authDomain: env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
    projectId: env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
    storageBucket: env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
    messagingSenderId: env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
    appId: env.NEXT_PUBLIC_FIREBASE_APP_ID,
  },
  
  // Build configuration
  build: {
    mode: env.NEXT_PUBLIC_BUILD_MODE || 'development',
  }
} as const;

// Consolidated Auth Configuration - single source of truth with context awareness
export const authConfig = {
  // Server-side auth properties (using server config)
  server: {
    get apiUrl() { return serverConfig.apiUrl; },
    get clientId() { return serverConfig.auth.oidcClientId; },
    get secret() { return serverConfig.auth.oidcSecret; },
    get callbackUrl() { return `${serverConfig.siteUrl}/api/auth/callback/epsx-backend`; },
    get authorizationEndpoint() { return `${serverConfig.apiUrl}/oauth/authorize`; },
  },
  
  // Client-side auth properties (using client config)  
  client: {
    get apiUrl() { return clientConfig.apiUrl; },
    get clientId() { return clientConfig.auth.clientId; },
    get callbackUrl() { return `${clientConfig.appUrl}/api/auth/callback/epsx-backend`; },
    get authorizationEndpoint() { return `${clientConfig.apiUrl}/oauth/authorize`; },
  },
  
  // Common auth paths
  callbackPath: '/api/auth/callback/epsx-backend',
  signinPath: '/api/auth/signin/epsx-backend',
  signoutPath: '/api/auth/signout',
} as const;

// Consolidated Feature Flags - single source of truth
export const featureFlags = {
  // Feature enablement flags
  UNIFIED_USER_MANAGEMENT: env.NEXT_PUBLIC_ENABLE_UNIFIED_USERS === 'true',
  SERVER_COMPONENTS: env.NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS === 'true',
  NEW_NAVIGATION: env.NEXT_PUBLIC_ENABLE_NEW_NAV === 'true',
  BUNDLE_OPTIMIZATION: env.NEXT_PUBLIC_ENABLE_BUNDLE_OPT === 'true',
  
  // Environment flags
  DEV_MODE: env.NODE_ENV === 'development',
  PROD_MODE: env.NODE_ENV === 'production',
  
  // Rollout percentages
  rolloutPercentages: {
    unified_user_management: parseInt(env.NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS || '0'),
    server_components: parseInt(env.NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS || '0'),
    new_navigation: parseInt(env.NEXT_PUBLIC_ROLLOUT_NEW_NAV || '0'),
  }
} as const;

// Context-aware API configuration
export const apiConfig = {
  // Get appropriate URL based on context (server vs client)
  getUrl: (context: 'server' | 'client' = 'server') => {
    return context === 'server' ? serverConfig.apiUrl : clientConfig.apiUrl;
  },
  
  // Common API settings
  timeout: 30000,
  retryAttempts: 3,
  
  // Default headers for API requests
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  }
} as const;

// Runtime validation for client/server context misuse (development only)
if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  const serverOnlyVars = ['BACKEND_URL', 'NEXTAUTH_SECRET', 'OIDC_CLIENT_SECRET'];
  serverOnlyVars.forEach(varName => {
    if (process.env[varName]) {
      console.error(`🚨 Security Warning: Server-only variable ${varName} accessed on client!`);
    }
  });
  
  // Validate proper usage of configuration objects
  if (typeof serverConfig.auth.secret === 'undefined') {
    console.warn('⚠️  Missing server authentication configuration');
  }
  if (typeof clientConfig.apiUrl === 'undefined') {
    console.warn('⚠️  Missing client API configuration');
  }
}