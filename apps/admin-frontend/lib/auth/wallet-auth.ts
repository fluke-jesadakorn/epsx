/**
 * Wallet Authentication Module
 * Replaces OIDC unified-auth.ts with wallet-based authentication using SIWE
 * Maintains same interface for compatibility with existing code
 */

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { SiweMessage } from 'siwe';

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
    
    console.log('🔍 Wallet session check:', {
      walletAddress: walletAddress ? 'present' : 'missing',
      nonce: nonce ? 'present' : 'missing',
      signature: signature ? 'present' : 'missing',
      message: message ? 'present' : 'missing',
      expiresAt: expiresAt ? 'present' : 'missing'
    });
    
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
  } catch (error) {
    console.error('❌ Failed to get wallet session from cookies:', error);
    return null;
  }
}

/**
 * Validate wallet session with backend
 */
export async function validateWalletWithBackend(session: WalletSession): Promise<WalletUser | null> {
  try {
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    const response = await fetch(`${backendUrl}/api/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        wallet_address: session.wallet_address,
        signature: session.signature,
        nonce: session.nonce,
        message: session.message
      })
    });
    
    if (!response.ok) {
      console.error('❌ Wallet validation failed:', response.status);
      return null;
    }
    
    const userInfo = await response.json();
    
    return {
      sub: userInfo.wallet_address, // Use wallet address as subject
      wallet_address: userInfo.wallet_address,
      email: userInfo.email,
      name: userInfo.name,
      permissions: userInfo.permissions || [],
      platform_context: userInfo.platform_context
    };
  } catch (error) {
    console.error('❌ Wallet validation error:', error);
    return null;
  }
}

/**
 * Set wallet authentication session in cookies
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
 */
export function hasAdminAccess(user: WalletUser | undefined): boolean {
  if (!user || !user.permissions || !Array.isArray(user.permissions)) {
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
 */
export function hasPermission(user: WalletUser | undefined, requiredPermission: string): boolean {
  if (!user || !user.permissions || !Array.isArray(user.permissions)) {
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
 */
export function getPermissionsByPlatform(permissions: string[], platform: string): string[] {
  return permissions.filter(permission => 
    permission.startsWith(`${platform}:`) || 
    permission === 'admin:*:*'
  );
}

/**
 * Check if permissions are expiring soon (for embedded timestamps)
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
      console.log('📝 No wallet session found');
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
      console.log('📝 Wallet session expired');
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
    } catch (error) {
      console.log('📝 Invalid SIWE message format');
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
      console.log('📝 Wallet validation with backend failed');
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
      console.log('📝 Wallet lacks admin permissions');
      return {
        isAuthenticated: true,
        isLoggedIn: true,
        user,
        hasAdminAccess: false,
        error: 'Insufficient admin permissions'
      };
    }
    
    console.log('✅ Valid admin wallet session established for:', user.wallet_address);
    
    return {
      isAuthenticated: true,
      isLoggedIn: true,
      user,
      hasAdminAccess: true,
      expiresAt: walletSession.expires_at
    };
    
  } catch (error) {
    console.error('❌ Wallet session validation error:', error);
    return {
      isAuthenticated: false,
      isLoggedIn: false,
      user: null,
      hasAdminAccess: false,
      error: error instanceof Error ? error.message : 'Session validation failed'
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
 */
export async function generateWalletNonce(walletAddress: string): Promise<string> {
  try {
    const response = await fetch('/api/auth/web3/challenge', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address: walletAddress }),
    });

    if (!response.ok) {
      throw new Error('Failed to generate nonce');
    }

    const { nonce } = await response.json();
    return nonce;
  } catch (error) {
    console.error('❌ Failed to generate wallet nonce:', error);
    throw error;
  }
}

/**
 * Verify wallet signature and create session
 */
export async function verifyWalletSignature(data: {
  wallet_address: string;
  signature: string;
  nonce: string;
  message: string;
}): Promise<AuthenticationResult> {
  try {
    const response = await fetch('/api/auth/web3/verify', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
      credentials: 'include',
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Verification failed' }));
      return { success: false, error: errorData.error };
    }

    const result = await response.json();
    
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
    
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Wallet authentication failed'
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

export async function setAuthTokens(data: any): Promise<void> {
  // This function is deprecated in wallet auth
  console.warn('setAuthTokens is deprecated in wallet authentication');
}

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