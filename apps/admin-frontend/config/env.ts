// Environment configuration for admin-frontend
// All variables read directly from process.env - no shared dependencies

import { z } from 'zod';

// Dynamic URL construction for admin frontend
function getAdminUrl(): string {
  if (process.env.NEXT_PUBLIC_ADMIN_URL) {
    return process.env.NEXT_PUBLIC_ADMIN_URL;
  }
  
  // Production should always provide NEXT_PUBLIC_ADMIN_URL
  if (process.env.NODE_ENV === 'production') {
    throw new Error('NEXT_PUBLIC_ADMIN_URL is required in production environment');
  }
  
  // Development fallback
  return 'http://localhost:3001';
}

function getBackendUrl(): string {
  if (process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL) {
    return process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || '';
  }
  
  // Production should always provide backend URL
  if (process.env.NODE_ENV === 'production') {
    throw new Error('NEXT_PUBLIC_BACKEND_URL or BACKEND_URL is required in production environment');
  }
  
  // Development fallback
  return 'http://localhost:8080';
}

function getFrontendUrl(): string {
  if (process.env.NEXT_PUBLIC_APP_URL) {
    return process.env.NEXT_PUBLIC_APP_URL;
  }
  
  // Production should always provide frontend URL
  if (process.env.NODE_ENV === 'production') {
    throw new Error('NEXT_PUBLIC_APP_URL is required in production environment');
  }
  
  // Development fallback
  return 'http://localhost:3000';
}

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'staging', 'production']).default('development'),
  PORT: z.string().transform(Number).default(3001),
  ADMIN_URL: z.string().url().default(getAdminUrl()),
  BACKEND_URL: z.string().url().default(getBackendUrl()),
  NEXT_PUBLIC_BACKEND_URL: z.string().url().default(getBackendUrl()),
  NEXT_PUBLIC_ADMIN_URL: z.string().url().default(getAdminUrl()),
  NEXT_PUBLIC_APP_URL: z.string().url().default(getFrontendUrl()),
  NEXT_PUBLIC_OAUTH_CLIENT_ID: z.string().optional(),
  NEXTAUTH_SECRET: z.string().min(1).default('dev-secret-key-32-chars-minimum'),
  OIDC_CLIENT_ID: z.string().min(1).default('epsx-admin'),
  OIDC_CLIENT_SECRET: z.string().min(1).default('dev-client-secret'),
  
  // Feature flags
  NEXT_PUBLIC_ENABLE_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ENABLE_NEW_NAV: z.string().optional(),
  NEXT_PUBLIC_ENABLE_BUNDLE_OPT: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS: z.string().optional(),
  NEXT_PUBLIC_ROLLOUT_NEW_NAV: z.string().optional(),
});

export const env = envSchema.parse(process.env);

// Consolidated auth configuration - single source of truth
export const authConfig = {
  appUrl: env.NEXT_PUBLIC_ADMIN_URL,
  apiUrl: env.NEXT_PUBLIC_BACKEND_URL,
  clientId: 'epsx-admin', // Fixed to correct client ID registered in backend
  callbackPath: '/api/auth/callback/epsx-backend',
  get callbackUrl() {
    return `${this.appUrl}${this.callbackPath}`;
  },
  get authorizationEndpoint() {
    return `${this.apiUrl}/oauth/authorize`;
  }
};

// Consolidated feature flags configuration - single source of truth
export const featureFlags = {
  // Unified User Management Hub
  UNIFIED_USER_MANAGEMENT: env.NEXT_PUBLIC_ENABLE_UNIFIED_USERS === 'true',
  
  // Server Components Migration
  SERVER_COMPONENTS: env.NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS === 'true',
  
  // New Navigation and URL Structure  
  NEW_NAVIGATION: env.NEXT_PUBLIC_ENABLE_NEW_NAV === 'true',
  
  // Performance Optimizations
  BUNDLE_OPTIMIZATION: env.NEXT_PUBLIC_ENABLE_BUNDLE_OPT === 'true',
  
  // Development and Testing
  DEV_MODE: env.NODE_ENV === 'development',

  // Rollout percentages
  rolloutPercentages: {
    unified_user_management: parseInt(env.NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS || '0'),
    server_components: parseInt(env.NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS || '0'),
    new_navigation: parseInt(env.NEXT_PUBLIC_ROLLOUT_NEW_NAV || '0'),
  }
} as const;