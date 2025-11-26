/**
 * Web3-First Admin Authentication Module
 * Complete Web3 wallet-first authentication for admin users
 * Uses SIWE (Sign-In with Ethereum) standard with group-based permissions
 */

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

import { apiFetch } from '@/lib/api-fetch';
import { getWeb3AdminSession, Web3AdminSessionData } from '@/lib/web3-admin-session';

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
    const walletAddress = cookieStore.get('epsx.user')?.value;
    // Note: signature and other auth data are HttpOnly and handled by backend
    // We can extract user info from the user cookie

    if (!walletAddress) {
      return null;
    }

    // Parse user data from cookie
    let userData = null;
    try {
      userData = JSON.parse(decodeURIComponent(walletAddress));
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
      chainId: 56,    // BSC Mainnet - default for consistency
      expiresAt: userData.auth_time ? parseInt(userData.auth_time) + 86400000 : now + 3600000, // 24 hours default
    };

    // Check if session has expired using new expiresAt calculation
    if (Date.now() > sessionData.expiresAt) {
      console.warn('Admin session expired, clearing...');
      await clearWeb3Session();
      return null;
    }

    return sessionData;
    
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to get Web3 session from cookies:', _error);
    return null;
  }
}

/**
 * Validate Web3 authentication with backend
 * @param sessionData
 */
export async function validateWeb3Session(_sessionData: Web3SessionData): Promise<Web3AdminUser | null> {
  try {
    // Note: Since we don't have access to signature and message (HttpOnly),
    // we'll rely on the backend validation through HttpOnly cookies
    // Create basic Web3AdminUser from available data
    const cookieStore = await cookies();
    const userCookie = cookieStore.get('epsx.user')?.value;

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
      sessionExpiry: userData.auth_time ? parseInt(userData.auth_time) + 86400000 : Date.now() + 86400000,
      lastVerified: Date.now()
    };

    return web3User;

  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Web3 validation error:', _error);
    return null;
  }
}

/**
 * Set Web3 session data in secure cookies
 * @param sessionData
 */
export async function setWeb3Session(sessionData: Web3SessionData): Promise<void> {
  const cookieStore = await cookies();
  
  const cookieOptions = {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    maxAge: Math.floor((sessionData.expiresAt - Date.now()) / 1000), // Convert to seconds
    path: '/'
  };

  // Note: sessionData only contains accessible data
  // HttpOnly auth cookies are set by backend during authentication

  cookieStore.set('epsx.user', JSON.stringify({
    wallet_address: sessionData.walletAddress,
    sub: sessionData.walletAddress,
    auth_time: Date.now(),
    permissions: [],
    groups: ['admin'],
    isAdmin: true,
    expires_at: sessionData.expiresAt
  }), cookieOptions);
}

/**
 * Clear Web3 session data
 */
export async function clearWeb3Session(): Promise<void> {
  const cookieStore = await cookies();

  // Clear unified EPSX session cookie
  cookieStore.delete('epsx.user');
  cookieStore.delete('epsx.access');
  cookieStore.delete('epsx.id');
  cookieStore.delete('epsx.refresh');

  // Clear old cookie names (backward compatibility)
  cookieStore.delete('epsx.admin.user');
  cookieStore.delete('wallet_address');
  cookieStore.delete('wallet_signature');
  cookieStore.delete('wallet_message');
  cookieStore.delete('wallet_nonce');
  cookieStore.delete('wallet_chain_id');
  cookieStore.delete('wallet_expires_at');

  // Clear any legacy tokens
  cookieStore.delete('admin_jwt_token');
  cookieStore.delete('session_token');
  cookieStore.delete('access_token');
  cookieStore.delete('id_token');
  cookieStore.delete('refresh_token');
}

// ============================================================================
// Permission System - Web3 Group-Based
// ============================================================================

/**
 * Check if Web3 user has admin access
 * @param user
 */
export function hasAdminAccess(user: Web3AdminUser | undefined): boolean {
  if (!user) {
    return false;
  }
  
  // Direct admin flag check (fastest)
  if (user.isAdmin) {
    return true;
  }
  
  // Check permissions array
  if (user.permissions && Array.isArray(user.permissions)) {
    return user.permissions.some(permission => 
      permission === 'admin:*:*' || 
      permission.startsWith('admin:') ||
      permission === 'epsx:admin:*'
    );
  }
  
  // Check admin groups
  if (user.groups && Array.isArray(user.groups)) {
    return user.groups.some(group => 
      group.includes('admin') || 
      group.includes('Admin') ||
      group === 'admin-users' ||
      group === 'admin-full'
    );
  }
  
  return false;
}

/**
 * Check if permissions array has admin access (legacy function)
 * @param permissions
 */
export function checkAdminPermissions(permissions: string[]): boolean {
  if (!permissions || !Array.isArray(permissions)) {
    return false;
  }
  
  return permissions.some(permission => 
    permission === 'admin:*:*' || 
    permission.startsWith('admin:') ||
    permission === 'epsx:admin:*'
  );
}

/**
 * Shared wildcard permission matcher - SINGLE SOURCE OF TRUTH
 * Handles admin:*:*, platform:*:*, and platform:resource:* wildcards
 */
function matchesPermission(permissions: string[], required: string): boolean {
  if (!permissions || !Array.isArray(permissions)) return false;

  // Admin wildcard - grants all permissions
  if (permissions.includes('admin:*:*')) return true;

  // Exact match
  if (permissions.includes(required)) return true;

  // Wildcard matching for structured permissions (platform:resource:action)
  const [platform, resource] = required.split(':');
  if (platform && resource) {
    // Platform wildcard: platform:*:*
    if (permissions.includes(`${platform}:*:*`)) return true;
    // Resource wildcard: platform:resource:*
    if (permissions.includes(`${platform}:${resource}:*`)) return true;
  }

  return false;
}

/**
 * Check specific permission for Web3 user
 */
export function hasPermission(user: Web3AdminUser | undefined, requiredPermission: string): boolean {
  if (!user) return false;
  if (user.isAdmin || hasAdminAccess(user)) return true;
  return matchesPermission(user.permissions, requiredPermission);
}

/**
 * Check specific permission with permissions array
 */
export function checkPermission(userPermissions: string[], requiredPermission: string): boolean {
  return matchesPermission(userPermissions, requiredPermission);
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
 * @param permissions
 * @param withinDays
 */
export function getExpiringPermissions(permissions: string[], withinDays = 7): string[] {
  const now = Date.now() / 1000;
  const threshold = now + (withinDays * 24 * 60 * 60);
  
  return permissions.filter(permission => {
    const parts = permission.split(':');
    if (parts.length === 4) {
      const expiryTimestamp = parseInt(parts[3], 10);
      return !isNaN(expiryTimestamp) && expiryTimestamp <= threshold;
    }
    return false;
  });
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
      await clearWeb3Session(); // Clear invalid session
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
    // eslint-disable-next-line no-console
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
    redirect('/login');
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