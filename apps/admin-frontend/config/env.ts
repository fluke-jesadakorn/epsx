// Environment configuration for admin-frontend
// All variables read directly from process.env - no shared dependencies

import { z } from 'zod';

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'staging', 'production']).default('development'),
  PORT: z.string().transform(Number).default(3001),
  NEXTAUTH_URL: z.string().url(),
  BACKEND_URL: z.string().url().default('https://api.epsx.io'),
  NEXT_PUBLIC_BACKEND_URL: z.string().url().default('https://api.epsx.io'),
  NEXT_PUBLIC_ADMIN_URL: z.string().url().default('https://admin.epsx.io'),
  NEXT_PUBLIC_APP_URL: z.string().url().default('https://epsx.io'),
  NEXT_PUBLIC_API_URL: z.string().url().optional(),
  NEXT_PUBLIC_OAUTH_CLIENT_ID: z.string().optional(),
  NEXTAUTH_SECRET: z.string().min(1),
  OIDC_CLIENT_ID: z.string().min(1),
  OIDC_CLIENT_SECRET: z.string().min(1),
  
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
  apiUrl: env.NEXT_PUBLIC_API_URL || env.NEXT_PUBLIC_BACKEND_URL,
  clientId: env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend',
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