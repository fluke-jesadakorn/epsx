/**
 * Centralized URL Resolution Utility for EPSX Platform
 * 
 * Provides context-aware URL resolution to eliminate hardcoded URLs
 * and inconsistent fallback patterns across the codebase.
 * 
 * Key Features:
 * - Context-aware resolution (server vs client)
 * - Environment-specific defaults
 * - Consistent fallback strategy
 * - Type-safe URL construction
 */

import { serverEnv, clientEnv, isDev, isProd, isStaging } from '../env/schema';

// ============================================================================
// Modern Enum System for Type-Safe URL Resolution
// ============================================================================

/**
 * URL Context - Whether the URL is needed on server-side or client-side
 */
export enum URLContext {
  SERVER = 'server',
  CLIENT = 'client'
}

/**
 * Service Types - Available services in the EPSX platform
 */
export enum Service {
  BACKEND = 'backend',
  FRONTEND = 'frontend',
  ADMIN = 'admin'
}

/**
 * OIDC Endpoints - OAuth 2.0 / OpenID Connect endpoints
 */
export enum OIDCEndpoint {
  AUTHORIZE = 'authorize',
  TOKEN = 'token',
  USERINFO = 'userinfo',
  JWKS = 'jwks',
  REVOKE = 'revoke',
  INTROSPECT = 'introspect'
}

/**
 * Environment Types - Runtime environments
 */
export enum Environment {
  DEVELOPMENT = 'development',
  STAGING = 'staging',
  PRODUCTION = 'production'
}

/**
 * Common API Paths - Frequently used API endpoints
 */
export enum APIPath {
  // Authentication paths
  AUTH_SESSION = 'api/auth/session',
  AUTH_LOGOUT = 'api/auth/logout',
  AUTH_CALLBACK = 'api/auth/callback/epsx-backend',
  
  // API paths
  HEALTH = 'health',
  USERS = 'api/v1/users',
  NOTIFICATIONS = 'api/v1/notifications',
  ANALYTICS = 'api/v1/analytics',
  
  // Admin paths
  ADMIN = 'api/admin',
  ADMIN_USERS = 'api/admin/users',
  ADMIN_PERMISSIONS = 'api/admin/permissions'
}

// Legacy type for backward compatibility - will be deprecated
export type URLContextLegacy = 'server' | 'client';

/**
 * Core URL Resolution Functions
 * These functions provide the primary URL resolution logic for all services
 */

/**
 * Get the backend API URL with context-aware resolution
 * @param context - Whether this is for server-side or client-side usage (supports enum or string)
 * @returns The appropriate backend URL for the given context
 */
export function getBackendUrl(context: URLContext | URLContextLegacy = URLContext.SERVER): string {
  const resolvedContext = typeof context === 'string' ? context : context;
  
  if (resolvedContext === 'server') {
    // Server-side: Use server-only environment variables
    return serverEnv.BACKEND_URL || getDefaultBackendUrl();
  } else {
    // Client-side: Use client-accessible environment variables
    return clientEnv.NEXT_PUBLIC_BACKEND_URL || getDefaultBackendUrl();
  }
}

/**
 * Get the frontend application URL with context-aware resolution
 * @param context - Whether this is for server-side or client-side usage (supports enum or string)
 * @returns The appropriate frontend URL for the given context
 */
export function getFrontendUrl(context: URLContext | URLContextLegacy = URLContext.SERVER): string {
  const resolvedContext = typeof context === 'string' ? context : context;
  
  if (resolvedContext === 'server') {
    // Server-side: Use server-only environment variables
    return serverEnv.FRONTEND_URL || getDefaultFrontendUrl();
  } else {
    // Client-side: Use client-accessible environment variables
    return clientEnv.NEXT_PUBLIC_APP_URL || getDefaultFrontendUrl();
  }
}

/**
 * Get the admin frontend URL with context-aware resolution
 * @param context - Whether this is for server-side or client-side usage (supports enum or string)
 * @returns The appropriate admin frontend URL for the given context
 */
export function getAdminUrl(context: URLContext | URLContextLegacy = URLContext.SERVER): string {
  const resolvedContext = typeof context === 'string' ? context : context;
  
  if (resolvedContext === 'server') {
    // Server-side: Use server-only environment variables
    return serverEnv.ADMIN_FRONTEND_URL || getDefaultAdminUrl();
  } else {
    // Client-side: Use client-accessible environment variables
    return clientEnv.NEXT_PUBLIC_ADMIN_URL || getDefaultAdminUrl();
  }
}

/**
 * Environment-specific default URL functions
 * These provide consistent fallback URLs based on the current environment
 */

function getDefaultBackendUrl(): string {
  if (isDev) return 'http://localhost:8080';
  if (isStaging) return 'https://staging-api.epsx.io';
  return 'https://api.epsx.io'; // Production default
}

function getDefaultFrontendUrl(): string {
  if (isDev) return 'http://localhost:3000';
  if (isStaging) return 'https://staging.epsx.io';
  return 'https://epsx.io'; // Production default
}

function getDefaultAdminUrl(): string {
  if (isDev) return 'http://localhost:3001';
  if (isStaging) return 'https://staging-admin.epsx.io';
  return 'https://admin.epsx.io'; // Production default
}

/**
 * Modern environment detection
 * @returns Current environment as enum
 */
export function getCurrentEnvironment(): Environment {
  if (isDev) return Environment.DEVELOPMENT;
  if (isStaging) return Environment.STAGING;
  return Environment.PRODUCTION;
}

// ============================================================================
// Modern URL Class - Type-Safe & Enum-Based
// ============================================================================

/**
 * Modern URL resolution class with enum-based type safety
 * Provides a clean, autocomplete-friendly API for all URL operations
 */
export class URL {
  /**
   * Get service URL with enum-based type safety
   * @param service - Service type (Backend, Frontend, Admin)
   * @param context - URL context (Server, Client)
   * @returns Service URL
   */
  static get(service: Service, context: URLContext = URLContext.SERVER): string {
    switch (service) {
      case Service.BACKEND:
        return getBackendUrl(context);
      case Service.FRONTEND:
        return getFrontendUrl(context);
      case Service.ADMIN:
        return getAdminUrl(context);
      default:
        throw new Error(`Unknown service: ${service}`);
    }
  }

  /**
   * Build OIDC endpoint URL with type safety
   * @param endpoint - OIDC endpoint type
   * @param context - URL context (Server, Client)
   * @returns Complete OIDC endpoint URL
   */
  static oidc(endpoint: OIDCEndpoint, context: URLContext = URLContext.SERVER): string {
    const baseUrl = getBackendUrl(context);
    return `${baseUrl}/oauth/${endpoint.valueOf()}`;
  }

  /**
   * Build API endpoint URL with type safety
   * @param service - Target service
   * @param path - API path (can be enum or string)
   * @param context - URL context (Server, Client)
   * @returns Complete API URL
   */
  static api(service: Service, path: APIPath | string, context: URLContext = URLContext.SERVER): string {
    const baseUrl = URL.get(service, context);
    const cleanPath = typeof path === 'string' ? path : path;
    const finalPath = cleanPath.startsWith('/') ? cleanPath.slice(1) : cleanPath;
    return `${baseUrl}/${finalPath}`;
  }

  /**
   * Get current environment
   * @returns Current environment enum
   */
  static currentEnvironment(): Environment {
    return getCurrentEnvironment();
  }

  /**
   * Validate URL format
   * @param url - URL to validate
   * @returns Whether URL is valid
   */
  static validate(url: string): boolean {
    try {
      new globalThis.URL(url);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Build callback URL for OAuth flows
   * @param service - Target service for callback
   * @param context - URL context (Server, Client)
   * @returns OAuth callback URL
   */
  static callback(service: Service, context: URLContext = URLContext.SERVER): string {
    return URL.api(service, APIPath.AUTH_CALLBACK, context);
  }
}

/**
 * Specialized URL Builders
 * These functions construct specific endpoint URLs for common use cases
 */

/**
 * OIDC endpoint URLs
 */
export const oidcUrls = {
  /**
   * Get the OAuth authorization endpoint
   * @param context - URL context (supports enum or string)
   */
  authorize: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getBackendUrl(context)}/oauth/authorize`,
    
  /**
   * Get the OAuth token endpoint
   * @param context - URL context (supports enum or string)
   */
  token: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getBackendUrl(context)}/oauth/token`,
    
  /**
   * Get the OAuth userinfo endpoint
   * @param context - URL context (supports enum or string)
   */
  userinfo: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getBackendUrl(context)}/oauth/userinfo`,
    
  /**
   * Get the JWKS endpoint
   * @param context - URL context (supports enum or string)
   */
  jwks: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getBackendUrl(context)}/oauth/jwks`,
};

/**
 * Callback URL builders
 */
export const callbackUrls = {
  /**
   * Get the frontend OAuth callback URL
   * @param context - URL context (supports enum or string)
   */
  frontend: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getFrontendUrl(context)}/api/auth/callback/epsx-backend`,
    
  /**
   * Get the admin OAuth callback URL
   * @param context - URL context (supports enum or string)
   */
  admin: (context: URLContext | URLContextLegacy = URLContext.SERVER) => 
    `${getAdminUrl(context)}/api/auth/callback/epsx-backend`,
};

/**
 * API endpoint builders
 */
export const apiUrls = {
  /**
   * Build API endpoint URL
   * @param path - API path (without leading slash)
   * @param context - URL context (supports enum or string)
   */
  backend: (path: string, context: URLContext | URLContextLegacy = URLContext.SERVER) => {
    const cleanPath = path.startsWith('/') ? path.slice(1) : path;
    return `${getBackendUrl(context)}/${cleanPath}`;
  },
  
  /**
   * Build frontend API endpoint URL
   * @param path - API path (without leading slash)
   * @param context - URL context (supports enum or string)
   */
  frontend: (path: string, context: URLContext | URLContextLegacy = URLContext.SERVER) => {
    const cleanPath = path.startsWith('/') ? path.slice(1) : path;
    return `${getFrontendUrl(context)}/${cleanPath}`;
  },
  
  /**
   * Build admin API endpoint URL
   * @param path - API path (without leading slash)
   * @param context - URL context (supports enum or string)
   */
  admin: (path: string, context: URLContext | URLContextLegacy = URLContext.SERVER) => {
    const cleanPath = path.startsWith('/') ? path.slice(1) : path;
    return `${getAdminUrl(context)}/${cleanPath}`;
  },
};

/**
 * URL validation and utility functions
 */

/**
 * Validate that a URL is properly formatted
 * @param url - URL to validate
 * @returns Whether the URL is valid
 */
export function isValidUrl(url: string): boolean {
  try {
    new globalThis.URL(url);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get the current environment context
 * @returns Environment information for debugging
 */
export function getEnvironmentInfo() {
  return {
    isDev,
    isProd,
    isStaging,
    defaultBackendUrl: getDefaultBackendUrl(),
    defaultFrontendUrl: getDefaultFrontendUrl(),
    defaultAdminUrl: getDefaultAdminUrl(),
  };
}

/**
 * Debug function to show all resolved URLs
 * Only available in development
 */
export function debugUrls(context: URLContext | URLContextLegacy = URLContext.SERVER) {
  if (!isDev) return {};
  
  return {
    context,
    backend: getBackendUrl(context),
    frontend: getFrontendUrl(context),
    admin: getAdminUrl(context),
    oidc: {
      authorize: oidcUrls.authorize(context),
      token: oidcUrls.token(context),
      userinfo: oidcUrls.userinfo(context),
      jwks: oidcUrls.jwks(context),
    },
    callbacks: {
      frontend: callbackUrls.frontend(context),
      admin: callbackUrls.admin(context),
    },
    environment: getEnvironmentInfo(),
  };
}

/**
 * Convenience exports for common patterns
 */
export const urls = {
  backend: getBackendUrl,
  frontend: getFrontendUrl,
  admin: getAdminUrl,
  oidc: oidcUrls,
  callbacks: callbackUrls,
  api: apiUrls,
  // Modern URL class for new code
  modern: URL,
} as const;

export default urls;