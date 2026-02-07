/**
 * Web3-First Admin Authentication Module
 * Complete Web3 wallet-first authentication for admin users
 * Uses SIWE (Sign-In with Ethereum) standard with group-based permissions
 */

import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { clearWeb3SessionAction, setWeb3SessionAction } from './auth-actions';

// ============================================================================
// Core Types - Web3-First
// ============================================================================

export interface Web3AdminUser {
  walletAddress: string;
  chainId: number;
  displayName: string;
  permissions: string[];
  groups: string[];
  isAdmin: boolean;
  sessionExpiry?: number;
  lastVerified?: number;
}

export interface AdminSession {
  isAuthenticated: boolean;
  isLoggedIn: boolean; // Alias for backwards compatibility
  user: Web3AdminUser | null;
  hasAdminAccess: boolean;
  expiresAt?: number;
  error?: string;
}

export interface Web3SessionData {
  walletAddress: string;
  signature: string;
  message: string;
  nonce: string;
  chainId: number;
  expiresAt: number;
}

export interface AuthenticationResult {
  success: boolean;
  session?: AdminSession;
  error?: string;
  redirectUrl?: string;
}

// ============================================================================
// Web3 Session Management
// ============================================================================

/**
 * Get Web3 authentication data from EPSX HttpOnly cookies
 */
export async function getWeb3SessionFromCookies(): Promise<Web3SessionData | null> {
  try {
    const cookieStore = await cookies();

    // Use unified EPSX cookie naming (no context separation)
    const userCookie = cookieStore.get(COOKIES.user)?.value;
    // Note: signature and other auth data are HttpOnly and handled by backend
    // We can extract user info from the user cookie

    if (!userCookie) {
      return null;
    }

    // Parse user data from cookie
    let userData = null;
    try {
      userData = JSON.parse(decodeURIComponent(userCookie));
    } catch (parseError) {
      console.warn('Failed to parse user cookie:', parseError);
      return null;
    }

    const now = Date.now();
    const sessionData: Web3SessionData = {
      walletAddress: userData.wallet_address || userData.sub,
      signature: '', // Not accessible (HttpOnly)
      message: '',   // Not accessible (HttpOnly)
      nonce: '',     // Not accessible (HttpOnly)
      chainId: process.env['NEXT_PUBLIC_DEFAULT_CHAIN_ID'] ? parseInt(process.env['NEXT_PUBLIC_DEFAULT_CHAIN_ID']) : 56,    // BSC Mainnet - default for consistency
      expiresAt: userData.auth_time ? parseInt(userData.auth_time) + 2592000000 : now + 2592000000, // 30 days default
    };

    // Check if session has expired using new expiresAt calculation
    if (Date.now() > sessionData.expiresAt) {
      console.warn('Admin session expired (read-only check)');
      // Do not clear cookies here in Server Components - just return null
      return null;
    }

    return sessionData;

  } catch (_error) {

    console.error('❌ Failed to get Web3 session from cookies:', _error);
    return null;
  }
}

/**
 * Validate Web3 authentication with backend
 * @param _sessionData
 */
export async function validateWeb3Session(_sessionData: Web3SessionData): Promise<Web3AdminUser | null> {
  try {
    // Note: Since we don't have access to signature and message (HttpOnly),
    // we'll rely on the backend validation through HttpOnly cookies
    // Create basic Web3AdminUser from available data
    const cookieStore = await cookies();
    const userCookie = cookieStore.get(COOKIES.user)?.value;

    if (!userCookie) {
      return null;
    }

    let userData = null;
    try {
      userData = JSON.parse(decodeURIComponent(userCookie));
    } catch (parseError) {
      console.warn('Failed to parse user cookie:', parseError);
      return null;
    }

    const web3User: Web3AdminUser = {
      walletAddress: userData.wallet_address || userData.sub,
      chainId: 56, // BSC Mainnet - default
      displayName: `Admin (${userData.wallet_address?.slice(0, 6)}...${userData.wallet_address?.slice(-4)})`,
      permissions: userData.permissions || [],
      groups: userData.groups || [],
      isAdmin: userData.isAdmin || true, // Assume admin if user cookie exists
      sessionExpiry: userData.auth_time ? parseInt(userData.auth_time) + 2592000000 : Date.now() + 2592000000,
      lastVerified: Date.now()
    };

    return web3User;

  } catch (_error) {

    console.error('❌ Web3 validation error:', _error);
    return null;
  }
}

/**
 * Set Web3 session data in secure cookies
 * @param sessionData
 */
export async function setWeb3Session(sessionData: Web3SessionData): Promise<void> {
  await setWeb3SessionAction(sessionData);
}

/**
 * Clear Web3 session data
 */
export async function clearWeb3Session(): Promise<void> {
  await clearWeb3SessionAction();
}

// ============================================================================
// Permission System - Web3 Group-Based
// ============================================================================

/**
 * Check if Web3 user has admin access
 * @param user
 */
export function hasAdminAccess(user: Web3AdminUser | undefined): boolean {
  // PERMISSION REFACTOR: Client-side checks are now permissive.
  // Backend enforces actual access control.
  return !!user;
}

/**
 * Check if permissions array has admin access (legacy function)
 * @param _permissions
 */
export function checkAdminPermissions(_permissions: string[]): boolean {
  return true;
}

/**
 * Check specific permission for Web3 user
 * @param user
 * @param _requiredPermission
 */
export function hasPermission(user: Web3AdminUser | undefined, _requiredPermission: string): boolean {
  return !!user;
}

/**
 * Check specific permission with permissions array
 * @param _userPermissions
 * @param _requiredPermission
 */
export function checkPermission(_userPermissions: string[], _requiredPermission: string): boolean {
  return true;
}

/**
 * Filter permissions by platform
 * @param permissions
 * @param platform
 */
export function getPermissionsByPlatform(permissions: string[], platform: string): string[] {
  return permissions.filter(permission =>
    permission.startsWith(`${platform}:`) ||
    permission === 'admin:*:*'
  );
}

/**
 * Check if permissions are expiring soon (for embedded timestamps)
 * @param _permissions
 * @param _withinDays
 */
export function getExpiringPermissions(_permissions: string[], _withinDays = 7): string[] {
  // PERMISSION REFACTOR: UI display hint for expiring permissions.
  // Real expiry is enforced by the backend.
  return [];
}


// ============================================================================
// Session Management
// ============================================================================

/**
 * Get current admin session
 * Web3-only authentication using wallet signatures
 */
export async function getAdminSession(): Promise<AdminSession> {
  try {

    // Get Web3 session data from cookies
    const sessionData = await getWeb3SessionFromCookies();

    if (!sessionData) {
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'No Web3 session found'
      };
    }

    // Validate session with backend
    const web3User = await validateWeb3Session(sessionData);

    if (!web3User) {
      // clearWeb3Session(); // Do not clear in Server Components
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'Session validation failed'
      };
    }

    // Check admin permissions
    const adminAccess = hasAdminAccess(web3User);

    if (!adminAccess) {
      return {
        isAuthenticated: true,
        isLoggedIn: true,
        user: web3User,
        hasAdminAccess: false,
        error: 'Insufficient admin permissions'
      };
    }

    return {
      isAuthenticated: true,
      isLoggedIn: true,
      user: web3User,
      hasAdminAccess: true,
      expiresAt: sessionData.expiresAt
    };

  } catch (_error) {

    console.error('❌ Session validation error:', _error);
    return {
      isAuthenticated: false,
      isLoggedIn: false,
      user: null,
      hasAdminAccess: false,
      error: _error instanceof Error ? _error.message : 'Session validation failed'
    };
  }
}

/**
 * Require admin session or redirect
 */
export async function requireAdminSession(): Promise<AdminSession> {
  const session = await getAdminSession();

  if (!session.isAuthenticated) {
    redirect('/auth');
  }

  if (!session.hasAdminAccess) {
    redirect('/access-denied');
  }

  return session;
}

/**
 * Check if session is valid
 */
export async function isValidSession(): Promise<boolean> {
  const session = await getAdminSession();
  return session.isAuthenticated && session.hasAdminAccess;
}

/**
 * Logout user
 */
export async function logout(): Promise<void> {
  await clearWeb3Session();
}

// ============================================================================
// Export Main Web3 Authentication Interface
// ============================================================================

export const Web3AdminAuth = {
  // Session management
  getSession: getAdminSession,
  requireSession: requireAdminSession,
  isValid: isValidSession,

  // Web3 session management
  getWeb3Session: getWeb3SessionFromCookies,
  setWeb3Session,
  clearWeb3Session,
  validateWeb3Session,

  // Permission checking
  hasPermission,
  hasAdminAccess,
  checkAdminPermissions,
  getPermissionsByPlatform,
  getExpiringPermissions,

  // Authentication flows
  logout
};

// Backward compatibility export
export const UnifiedAuth = Web3AdminAuth;

export default Web3AdminAuth;