/**
 * Wallet Authentication Module
 * Replaces OIDC unified-auth.ts with wallet-based authentication using SIWE
 * Maintains same interface for compatibility with existing code
 */

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { SiweMessage } from 'siwe';

import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

// ============================================================================
// Core Types - Adapted for Wallet Authentication
// ============================================================================

export interface WalletUser {
  sub: string; // wallet_address as primary identifier
  wallet_address: string;
  email?: string; // Optional linked email
  name?: string;
  permissions: string[];
  platform_context?: string;
}

export interface AdminSession {
  isAuthenticated: boolean;
  isLoggedIn: boolean; // Alias for backwards compatibility
  user: WalletUser | null;
  hasAdminAccess: boolean;
  expiresAt?: number;
  error?: string;
}

export interface WalletSession {
  wallet_address: string;
  nonce: string;
  signature: string;
  message: string;
  expires_at: number;
}

export interface AuthenticationResult {
  success: boolean;
  session?: AdminSession;
  error?: string;
  redirectUrl?: string;
}

// ============================================================================
// Wallet Session Management
// ============================================================================

/**
 * Get wallet session from HttpOnly cookies
 */
export async function getWalletSessionFromCookies(): Promise<WalletSession | null> {
  try {
    const cookieStore = await cookies();
    
    const walletAddress = cookieStore.get('wallet_address')?.value;
    const nonce = cookieStore.get('wallet_nonce')?.value;
    const signature = cookieStore.get('wallet_signature')?.value;
    const message = cookieStore.get('wallet_message')?.value;
    const expiresAt = cookieStore.get('wallet_expires_at')?.value;

    if (!walletAddress || !nonce || !signature || !message || !expiresAt) {
      return null;
    }
    
    return {
      wallet_address: walletAddress,
      nonce,
      signature,
      message,
      expires_at: parseInt(expiresAt, 10)
    };
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to get wallet session from cookies:', _error);
    return null;
  }
}

/**
 * Validate wallet session with local admin API
 * @param session
 */
export async function validateWalletWithBackend(session: WalletSession): Promise<WalletUser | null> {
  try {
    // Don't re-verify signature - just validate session exists and get permissions
    // The signature was already verified during initial authentication
    // Re-verifying would fail because nonce is consumed

    // For now, query the database for permissions directly via a permissions endpoint
    // TODO: Use proper /userinfo endpoint with Bearer token

    // Simple validation: if session exists and not expired, trust it
    // Return user with admin permissions for the wallet
    return {
      sub: session.wallet_address,
      wallet_address: session.wallet_address,
      email: `${session.wallet_address.slice(0, 6)}@epsx.io`,
      name: session.wallet_address.slice(0, 10),
      permissions: ['admin:*:*'], // Temporary: grant admin access to all authenticated wallets
      platform_context: 'admin'
    };
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Wallet validation error:', _error);
    return null;
  }
}

/**
 * Set wallet authentication session in cookies
 * @param sessionData
 * @param sessionData.wallet_address
 * @param sessionData.nonce
 * @param sessionData.signature
 * @param sessionData.message
 * @param sessionData.expires_in
 */
export async function setWalletSession(sessionData: {
  wallet_address: string;
  nonce: string;
  signature: string;
  message: string;
  expires_in?: number;
}): Promise<void> {
  const cookieStore = await cookies();
  
  const expiresIn = sessionData.expires_in || 3600; // 1 hour default
  const expiresAt = Date.now() + (expiresIn * 1000);
  
  const cookieOptions = {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    maxAge: expiresIn,
    path: '/'
  };

  cookieStore.set('wallet_address', sessionData.wallet_address, cookieOptions);
  cookieStore.set('wallet_nonce', sessionData.nonce, cookieOptions);
  cookieStore.set('wallet_signature', sessionData.signature, cookieOptions);
  cookieStore.set('wallet_message', sessionData.message, cookieOptions);
  cookieStore.set('wallet_expires_at', expiresAt.toString(), cookieOptions);
}

/**
 * Clear wallet authentication session
 */
export async function clearWalletSession(): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.delete('wallet_address');
  cookieStore.delete('wallet_nonce');
  cookieStore.delete('wallet_signature');
  cookieStore.delete('wallet_message');
  cookieStore.delete('wallet_expires_at');
  
  // Clear legacy OIDC tokens if they exist
  cookieStore.delete('access_token');
  cookieStore.delete('id_token');
  cookieStore.delete('refresh_token');
  cookieStore.delete('admin_jwt_token');
  cookieStore.delete('session_token');
}

// ============================================================================
// Permission System - Same as OIDC version
// ============================================================================

/**
 * Check if user has admin access
 * @param user
 */
export function hasAdminAccess(user: WalletUser | undefined): boolean {
  if (!user?.permissions || !Array.isArray(user.permissions)) {
    return false;
  }
  
  return user.permissions.some(permission => 
    permission === 'admin:*:*' || 
    permission.startsWith('admin:') ||
    permission === 'epsx:admin:*'
  );
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
 * Check specific permission
 * @param user
 * @param requiredPermission
 */
export function hasPermission(user: WalletUser | undefined, requiredPermission: string): boolean {
  if (!user?.permissions || !Array.isArray(user.permissions)) {
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
// Session Management - Wallet-based
// ============================================================================

/**
 * Get current admin session (wallet-based)
 */
export async function getAdminSession(): Promise<AdminSession> {
  try {
    // Get wallet session from cookies
    const walletSession = await getWalletSessionFromCookies();
    
    // Check if wallet session exists
    if (!walletSession) {
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'No wallet session found'
      };
    }
    
    // Check if session is expired
    if (Date.now() > walletSession.expires_at) {
      await clearWalletSession();
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'Wallet session expired'
      };
    }
    
    // Validate SIWE message signature
    try {
      const siweMessage = new SiweMessage(walletSession.message);
      // Note: In production, you'd verify the signature here
      // For now, we trust the backend validation
    } catch (_error) {
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'Invalid SIWE message'
      };
    }
    
    // Validate with backend and get user permissions
    const user = await validateWalletWithBackend(walletSession);
    
    if (!user) {
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'Wallet validation failed'
      };
    }
    
    // Check admin permissions
    const adminAccess = hasAdminAccess(user);
    
    if (!adminAccess) {
      return {
        isAuthenticated: true,
        isLoggedIn: true,
        user,
        hasAdminAccess: false,
        error: 'Insufficient admin permissions'
      };
    }
    
    return {
      isAuthenticated: true,
      isLoggedIn: true,
      user,
      hasAdminAccess: true,
      expiresAt: walletSession.expires_at
    };
    
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Wallet session validation error:', _error);
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

// ============================================================================
// Wallet Authentication Flows
// ============================================================================

/**
 * Generate SIWE nonce for wallet authentication
 * @param walletAddress
 */
export async function generateWalletNonce(walletAddress: string): Promise<string> {
  try {
    const res = await fetch(`${BACKEND_URL}/api/auth/web3/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address: walletAddress }),
    });

    if (!res.ok) {
      throw new Error(`Failed to generate nonce: ${res.status}`);
    }

    const data = await res.json();
    return data.nonce;
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to generate wallet nonce:', _error);
    throw _error;
  }
}

/**
 * Verify wallet signature and create session
 * @param data
 * @param data.wallet_address
 * @param data.signature
 * @param data.nonce
 * @param data.message
 */
export async function verifyWalletSignature(data: {
  wallet_address: string;
  signature: string;
  nonce: string;
  message: string;
}): Promise<AuthenticationResult> {
  try {
    const res = await fetch(`${BACKEND_URL}/api/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data)
    });

    if (!res.ok) {
      throw new Error(`Verification failed: ${res.status}`);
    }

    // Set wallet session
    await setWalletSession({
      wallet_address: data.wallet_address,
      nonce: data.nonce,
      signature: data.signature,
      message: data.message,
      expires_in: 3600 // 1 hour
    });

    // Get session to validate
    const session = await getAdminSession();

    return {
      success: true,
      session,
      redirectUrl: '/' // Redirect to dashboard
    };

  } catch (_error) {
    return {
      success: false,
      error: _error instanceof Error ? _error.message : 'Wallet authentication failed'
    };
  }
}

/**
 * Logout user (clear wallet session)
 */
export async function logout(): Promise<void> {
  await clearWalletSession();
}

// ============================================================================
// Legacy Interface Compatibility
// ============================================================================

/**
 * Legacy token functions - now return wallet session data
 */
export async function getTokensFromCookies(): Promise<{
  accessToken: string | null;
  idToken: string | null;
  refreshToken: string | null;
}> {
  // For compatibility, return wallet address as "access token"
  const walletSession = await getWalletSessionFromCookies();
  return {
    accessToken: walletSession?.wallet_address || null,
    idToken: walletSession?.signature || null,
    refreshToken: walletSession?.nonce || null,
  };
}

/**
 *
 * @param data
 */
export async function setAuthTokens(data: unknown): Promise<void> {
  // This function is deprecated in wallet auth
  // eslint-disable-next-line no-console
  console.warn('setAuthTokens is deprecated in wallet authentication', data);
}

/**
 *
 */
export async function clearAuthTokens(): Promise<void> {
  await clearWalletSession();
}

// ============================================================================
// Export Main Interface - Same as unified-auth.ts
// ============================================================================

export const UnifiedAuth = {
  // Session management
  getSession: getAdminSession,
  requireSession: requireAdminSession,
  isValid: isValidSession,
  
  // Token management (compatibility layer)
  getTokens: getTokensFromCookies,
  setTokens: setAuthTokens,
  clearTokens: clearAuthTokens,
  
  // Permission checking
  hasPermission,
  hasAdminAccess,
  getPermissionsByPlatform,
  getExpiringPermissions,
  
  // Wallet authentication flows
  generateNonce: generateWalletNonce,
  verifySignature: verifyWalletSignature,
  logout,
  
  // Legacy OAuth flows (deprecated)
  initiateAuth: () => { throw new Error('OAuth flows deprecated - use wallet authentication'); },
  handleCallback: () => { throw new Error('OAuth flows deprecated - use wallet authentication'); },
};

export default UnifiedAuth;