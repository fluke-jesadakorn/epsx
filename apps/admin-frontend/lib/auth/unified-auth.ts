/**
 * Unified Authentication Module
 * Consolidates all authentication logic into a single, consistent interface
 * Replaces: admin-oidc-auth.ts, admin-auth-helpers.ts, server-auth.ts, session.ts, helpers.ts
 */

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { URL, URLContext, OIDCEndpoint } from '../../../../shared/utils/url-resolver';

// ============================================================================
// Core Types
// ============================================================================

export interface OIDCUser {
  sub: string;
  email: string;
  name?: string;
  permissions: string[];
  platform_context?: string;
}

export interface AdminSession {
  isAuthenticated: boolean;
  isLoggedIn: boolean; // Alias for backwards compatibility
  user: OIDCUser | null;
  hasAdminAccess: boolean;
  expiresAt?: number;
  error?: string;
}

export interface TokenPair {
  access_token: string;
  id_token?: string;
  refresh_token?: string;
  expires_in?: number;
}

export interface AuthenticationResult {
  success: boolean;
  session?: AdminSession;
  error?: string;
  redirectUrl?: string;
}

// ============================================================================
// Token Management
// ============================================================================

/**
 * Get OIDC tokens from HttpOnly cookies
 */
export async function getTokensFromCookies(): Promise<{
  accessToken: string | null;
  idToken: string | null;
  refreshToken: string | null;
}> {
  try {
    const cookieStore = await cookies();
    
    const accessToken = cookieStore.get('access_token')?.value || null;
    const idToken = cookieStore.get('id_token')?.value || null;
    const refreshToken = cookieStore.get('refresh_token')?.value || null;
    
    console.log('🔍 Token check:', {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing',
      refreshToken: refreshToken ? 'present' : 'missing'
    });
    
    return { accessToken, idToken, refreshToken };
  } catch (error) {
    console.error('❌ Failed to get tokens from cookies:', error);
    return { accessToken: null, idToken: null, refreshToken: null };
  }
}

/**
 * Validate access token with backend
 */
export async function validateTokenWithBackend(accessToken: string): Promise<OIDCUser | null> {
  try {
    const userinfoEndpoint = URL.oidc(OIDCEndpoint.USERINFO, URLContext.CLIENT);
    
    const response = await fetch(userinfoEndpoint, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      console.error('❌ Token validation failed:', response.status);
      return null;
    }
    
    const userInfo = await response.json();
    
    return {
      sub: userInfo.sub,
      email: userInfo.email,
      name: userInfo.name,
      permissions: userInfo.permissions || [],
      platform_context: userInfo.platform_context
    };
  } catch (error) {
    console.error('❌ Token validation error:', error);
    return null;
  }
}

/**
 * Set authentication tokens in cookies
 */
export async function setAuthTokens(tokens: TokenPair): Promise<void> {
  const cookieStore = await cookies();
  
  const cookieOptions = {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    maxAge: tokens.expires_in || 3600, // 1 hour default
    path: '/'
  };

  cookieStore.set('access_token', tokens.access_token, cookieOptions);
  
  if (tokens.id_token) {
    cookieStore.set('id_token', tokens.id_token, cookieOptions);
  }
  
  if (tokens.refresh_token) {
    cookieStore.set('refresh_token', tokens.refresh_token, {
      ...cookieOptions,
      maxAge: 7 * 24 * 60 * 60 // 7 days for refresh token
    });
  }
}

/**
 * Clear authentication tokens
 */
export async function clearAuthTokens(): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.delete('access_token');
  cookieStore.delete('id_token');
  cookieStore.delete('refresh_token');
  
  // Clear legacy tokens
  cookieStore.delete('admin_jwt_token');
  cookieStore.delete('session_token');
}

// ============================================================================
// Permission System
// ============================================================================

/**
 * Check if user has admin access
 */
export function hasAdminAccess(user: OIDCUser | undefined): boolean {
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
export function hasPermission(user: OIDCUser | undefined, requiredPermission: string): boolean {
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
// Session Management
// ============================================================================

/**
 * Get current admin session
 */
export async function getAdminSession(): Promise<AdminSession> {
  try {
    // Get tokens from cookies
    const { accessToken, idToken } = await getTokensFromCookies();
    
    // Check if required tokens are present
    if (!accessToken || !idToken) {
      console.log('📝 No valid tokens found');
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'No authentication tokens found'
      };
    }
    
    // Validate token with backend
    const user = await validateTokenWithBackend(accessToken);
    
    if (!user) {
      console.log('📝 Token validation failed');
      return {
        isAuthenticated: false,
        isLoggedIn: false,
        user: null,
        hasAdminAccess: false,
        error: 'Token validation failed'
      };
    }
    
    // Check admin permissions
    const adminAccess = hasAdminAccess(user);
    
    if (!adminAccess) {
      console.log('📝 User lacks admin permissions');
      return {
        isAuthenticated: true,
        isLoggedIn: true,
        user,
        hasAdminAccess: false,
        error: 'Insufficient admin permissions'
      };
    }
    
    console.log('✅ Valid admin session established for:', user.email);
    
    return {
      isAuthenticated: true,
      isLoggedIn: true,
      user,
      hasAdminAccess: true
    };
    
  } catch (error) {
    console.error('❌ Session validation error:', error);
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
// Authentication Flows
// ============================================================================

/**
 * Initiate OIDC authentication flow
 */
export async function initiateAuth(): Promise<string> {
  const state = generateRandomState();
  const codeChallenge = generateCodeChallenge();
  
  const authUrl = new URL(`${process.env.NEXT_PUBLIC_BACKEND_URL}/oauth/authorize`);
  authUrl.searchParams.append('response_type', 'code');
  authUrl.searchParams.append('client_id', 'admin-frontend');
  authUrl.searchParams.append('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL}/api/auth/callback/epsx-backend`);
  authUrl.searchParams.append('scope', 'openid profile email admin:*:*');
  authUrl.searchParams.append('state', state);
  authUrl.searchParams.append('code_challenge', codeChallenge);
  authUrl.searchParams.append('code_challenge_method', 'S256');
  
  // Store state and code verifier in cookies for validation
  const cookieStore = await cookies();
  cookieStore.set('oauth_state', state, { httpOnly: true, maxAge: 600 }); // 10 minutes
  cookieStore.set('code_verifier', generateCodeVerifier(), { httpOnly: true, maxAge: 600 });
  
  return authUrl.toString();
}

/**
 * Handle authentication callback
 */
export async function handleAuthCallback(code: string, state: string): Promise<AuthenticationResult> {
  try {
    const cookieStore = await cookies();
    const storedState = cookieStore.get('oauth_state')?.value;
    const codeVerifier = cookieStore.get('code_verifier')?.value;
    
    // Validate state
    if (state !== storedState) {
      return { success: false, error: 'Invalid state parameter' };
    }
    
    // Exchange code for tokens
    const tokenResponse = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/oauth/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: 'admin-frontend',
        code,
        redirect_uri: `${process.env.NEXT_PUBLIC_APP_URL}/api/auth/callback/epsx-backend`,
        code_verifier: codeVerifier || ''
      })
    });
    
    if (!tokenResponse.ok) {
      return { success: false, error: 'Token exchange failed' };
    }
    
    const tokens = await tokenResponse.json();
    
    // Set tokens in cookies
    await setAuthTokens(tokens);
    
    // Clean up temporary cookies
    cookieStore.delete('oauth_state');
    cookieStore.delete('code_verifier');
    
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
      error: error instanceof Error ? error.message : 'Authentication failed'
    };
  }
}

/**
 * Logout user
 */
export async function logout(): Promise<void> {
  await clearAuthTokens();
}

// ============================================================================
// Utility Functions
// ============================================================================

function generateRandomState(): string {
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}

// PKCE functions now available from shared utilities

// ============================================================================
// Legacy Migration Support - Now handled by shared utilities
// ============================================================================

// ============================================================================
// Export Main Interface
// ============================================================================

export const UnifiedAuth = {
  // Session management
  getSession: getAdminSession,
  requireSession: requireAdminSession,
  isValid: isValidSession,
  
  // Token management
  getTokens: getTokensFromCookies,
  setTokens: setAuthTokens,
  clearTokens: clearAuthTokens,
  
  // Permission checking
  hasPermission,
  hasAdminAccess,
  getPermissionsByPlatform,
  getExpiringPermissions,
  
  // Authentication flows
  initiateAuth,
  handleCallback: handleAuthCallback,
  logout,
  
  // Legacy support - now handled by shared utilities
};

export default UnifiedAuth;