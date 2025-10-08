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
 * Get Web3 authentication data from HttpOnly cookies
 */
export async function getWeb3SessionFromCookies(): Promise<Web3SessionData | null> {
  try {
    const cookieStore = await cookies();
    
    const walletAddress = cookieStore.get('wallet_address')?.value;
    const signature = cookieStore.get('wallet_signature')?.value;
    const message = cookieStore.get('wallet_message')?.value;
    const nonce = cookieStore.get('wallet_nonce')?.value;
    const chainId = cookieStore.get('wallet_chain_id')?.value;
    const expiresAt = cookieStore.get('wallet_expires_at')?.value;
    
    if (!walletAddress || !signature || !message || !nonce || !chainId || !expiresAt) {
      return null;
    }
    
    const sessionData: Web3SessionData = {
      walletAddress,
      signature,
      message,
      nonce,
      chainId: parseInt(chainId, 10),
      expiresAt: parseInt(expiresAt, 10)
    };
    
    // Check if session has expired
    if (Date.now() > sessionData.expiresAt) {
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
export async function validateWeb3Session(sessionData: Web3SessionData): Promise<Web3AdminUser | null> {
  try {
    const validationResult = await apiFetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/verify`, {
      method: 'POST',
      body: JSON.stringify({
        walletAddress: sessionData.walletAddress,
        signature: sessionData.signature,
        message: sessionData.message,
        nonce: sessionData.nonce,
        chainId: sessionData.chainId
      })
    });

    if (!validationResult.valid || !validationResult.hasAdminAccess) {
      return null;
    }

    // Create Web3AdminUser from validation result
    const web3User: Web3AdminUser = {
      walletAddress: sessionData.walletAddress,
      chainId: sessionData.chainId,
      displayName: `Admin (${sessionData.walletAddress.slice(0, 6)}...${sessionData.walletAddress.slice(-4)})`,
      permissions: validationResult.permissions || [],
      groups: validationResult.groups || [],
      isAdmin: validationResult.hasAdminAccess,
      sessionExpiry: sessionData.expiresAt,
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

  cookieStore.set('wallet_address', sessionData.walletAddress, cookieOptions);
  cookieStore.set('wallet_signature', sessionData.signature, cookieOptions);
  cookieStore.set('wallet_message', sessionData.message, cookieOptions);
  cookieStore.set('wallet_nonce', sessionData.nonce, cookieOptions);
  cookieStore.set('wallet_chain_id', sessionData.chainId.toString(), cookieOptions);
  cookieStore.set('wallet_expires_at', sessionData.expiresAt.toString(), cookieOptions);
}

/**
 * Clear Web3 session data
 */
export async function clearWeb3Session(): Promise<void> {
  const cookieStore = await cookies();
  
  // Clear Web3 session cookies
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
 * Check specific permission for Web3 user
 * @param user
 * @param requiredPermission
 */
export function hasPermission(user: Web3AdminUser | undefined, requiredPermission: string): boolean {
  if (!user) {
    return false;
  }
  
  // Admin users have all permissions
  if (user.isAdmin || hasAdminAccess(user)) {
    return true;
  }
  
  if (!user.permissions || !Array.isArray(user.permissions)) {
    return false;
  }
  
  // Wildcard admin access
  if (user.permissions.includes('admin:*:*')) {
    return true;
  }
  
  // Exact match
  if (user.permissions.includes(requiredPermission)) {
    return true;
  }
  
  // Pattern matching for structured permissions
  const [platform, resource, action] = requiredPermission.split(':');
  
  // Check platform wildcard
  if (user.permissions.includes(`${platform}:*:*`)) {
    return true;
  }
  
  // Check resource wildcard
  if (user.permissions.includes(`${platform}:${resource}:*`)) {
    return true;
  }
  
  return false;
}

/**
 * Check specific permission (legacy function with permissions array)
 * @param userPermissions
 * @param requiredPermission
 */
export function checkPermission(userPermissions: string[], requiredPermission: string): boolean {
  if (!userPermissions || !Array.isArray(userPermissions)) {
    return false;
  }
  
  // Wildcard admin access
  if (userPermissions.includes('admin:*:*')) {
    return true;
  }
  
  // Exact match
  if (userPermissions.includes(requiredPermission)) {
    return true;
  }
  
  // Pattern matching for structured permissions
  const [platform, resource, action] = requiredPermission.split(':');
  
  // Check platform wildcard
  if (userPermissions.includes(`${platform}:*:*`)) {
    return true;
  }
  
  // Check resource wildcard
  if (userPermissions.includes(`${platform}:${resource}:*`)) {
    return true;
  }
  
  return false;
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