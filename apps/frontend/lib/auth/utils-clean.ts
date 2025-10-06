/**
 * Authentication Utilities (Backend-Only Permissions)
 *
 * IMPORTANT: This file contains ZERO permission checking logic.
 * All permission decisions are made by the backend.
 * Frontend only handles authentication state and displays backend errors.
 */

import { JWTPayload } from 'jose';

// ============================================================================
// JWT Interfaces (Read-Only)
// ============================================================================

export interface JWTUser {
  uid: string;
  email: string;
  scope?: string;  // OIDC standard scope claim (read-only, for display)
  iat?: number;
  exp?: number;
}

export interface EPSXJWTPayload extends JWTPayload {
  sub: string;
  email: string;
  name: string;
  scope: string;  // OIDC standard: "openid profile epsx:analytics:read"
  iat: number;
  exp: number;
}

// ============================================================================
// Backend Error Types
// ============================================================================

export interface PermissionError {
  code: number;
  message: string;
  reason?: string;
  required_permission?: string;
  upgrade_group?: string;
  upgrade_url?: string;
}

export interface ApiErrorResponse {
  success: false;
  error: PermissionError;
}

// ============================================================================
// Error Handling (Backend Errors Only)
// ============================================================================

/**
 * Handle API errors from backend
 * Use this to display permission errors returned from backend
 * @param error
 */
export function handleApiError(error: any): PermissionError | null {
  if (error?.response?.status === 403 || error?.response?.status === 401) {
    return error.response.data?.error ?? {
      code: error.response.status,
      message: 'Access denied',
      reason: 'Insufficient permissions'
    };
  }
  return null;
}

/**
 * Check if error is a permission error from backend
 * @param error
 */
export function isPermissionError(error: any): boolean {
  return error?.response?.status === 403;
}

/**
 * Get user-friendly error message from backend error
 * @param error
 */
export function getErrorMessage(error: PermissionError): string {
  if (error.upgrade_group) {
    return `${error.message}. Upgrade to ${error.upgrade_group} to access this feature.`;
  }
  return error.message ?? 'Access denied';
}

// ============================================================================
// Token Utilities (Read-Only)
// ============================================================================

/**
 * Check if user is authenticated (has valid token)
 * Does NOT check permissions - only authentication state
 */
export function isAuthenticated(): boolean {
  // Check if Bearer token exists in cookies/storage
  // This is just for UI state - backend validates the actual token
  if (typeof window === 'undefined') {return false;}

  const token = document.cookie
    .split('; ')
    .find(row => row.startsWith('access_token='))
    ?.split('=')[1];

  return !!token;
}

/**
 * Parse scope from JWT for display purposes only
 * NOT for permission validation - backend is the authority
 * @param jwt
 */
export function parseScopeFromJWT(jwt: EPSXJWTPayload): string[] {
  return jwt.scope
    .split(' ')
    .filter(s => s !== 'openid' && s !== 'profile');
}

// ============================================================================
// Display Helpers (No Permission Logic)
// ============================================================================

/**
 * Get display tier from scope (for UI display only)
 * NOT for access control - backend handles all permissions
 * @param scope
 */
export function getDisplayTierFromScope(scope: string): string {
  if (scope.includes('admin:')) {return 'Admin';}
  if (scope.includes('professional')) {return 'Professional';}
  if (scope.includes('premium')) {return 'Premium';}
  if (scope.includes('standard')) {return 'Standard';}
  return 'Basic';
}

// ============================================================================
// REMOVED FUNCTIONS (Now Handled by Backend)
// ============================================================================

// ❌ REMOVED: hasUserPermission() - Backend validates all permissions
// ❌ REMOVED: canAccessAnalytics() - Backend returns 403 if no access
// ❌ REMOVED: canAccessTrading() - Backend returns 403 if no access
// ❌ REMOVED: checkPermission() - Backend is the authority
// ❌ REMOVED: requirePermission() - Backend enforces requirements
// ❌ REMOVED: getAccessiblePlatforms() - Backend determines access
// ❌ REMOVED: getPrimaryPlatform() - Backend determines access
// ❌ REMOVED: getRankingLimit() - Backend enforces limits

// Frontend should ONLY:
// 1. Call APIs with Bearer token
// 2. Handle 403 errors from backend
// 3. Display backend error messages
// 4. Redirect to upgrade page if backend suggests
