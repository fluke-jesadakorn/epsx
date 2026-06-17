/**
 * Route Compatibility Layer for Admin Frontend
 * Bridges shared route constants with current backend implementation
 *
 * All routes now use /api/ prefix without versioning
 */

import { API_ROUTES } from '@/shared/config/route-constants';

// Auth routes implemented in backend
export const AUTH_ROUTES = {
  WEB3_CHALLENGE: '/api/auth/web3/challenge',
  WEB3_VERIFY: '/api/auth/web3/verify',
  WEB3_SESSION: '/api/auth/web3/session',
  WEB3_LOGOUT: '/api/auth/web3/logout',
} as const;

// Use standard routes from shared config
export const ROUTES = {
  AUTH: AUTH_ROUTES,
  // Add other route mappings as needed
} as const;

// Get standard routes from shared config
/**
 *
 */
export function getStandardRoutes() {
  return API_ROUTES;
}