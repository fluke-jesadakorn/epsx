/**
 * Route Compatibility Layer for Admin Frontend
 * Bridges shared route constants with current backend implementation
 *
 * This provides a temporary bridge until backend implements /api/v1/ routes
 */

import { API_ROUTES } from '@/shared/config/route-constants';

// Legacy routes currently implemented in backend (without /v1/)
export const LEGACY_ROUTES = {
  AUTH: {
    WEB3_CHALLENGE: '/api/auth/web3/challenge',
    WEB3_VERIFY: '/api/auth/web3/verify',
    WEB3_SESSION: '/api/auth/web3/session',
    WEB3_LOGOUT: '/api/auth/web3/logout',
  }
} as const;

// Use legacy routes for now, switch to shared routes when backend is ready
export const ROUTES = {
  AUTH: LEGACY_ROUTES.AUTH, // Use legacy until backend supports /v1/
  // Add other route mappings as needed
} as const;

// Future migration helper
export function migrateToStandardRoutes() {
  return API_ROUTES; // Will be switched when backend implements /v1/
}