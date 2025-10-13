/**
 * Server-side OAuth utilities for admin-frontend
 */

import { redirect } from 'next/navigation';

import { generateCodeVerifier, generateCodeChallenge, generateRandomString } from '../../../../shared/auth/pkce';

// OAuth authorization URL generation now handled by shared utilities
async function getAuthorizationUrl(): Promise<{ url: string; codeVerifier: string; state: string }> {
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateRandomString(32);
  return { url: '/auth', codeVerifier, state };
}

/**
 * Get server session with JWT verification
 */
export async function getServerSession() {
  try {
    const { getSessionFromJWT } = await import('./token');
    return await getSessionFromJWT();
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to get server session:', _error);
    return { isAuthenticated: false, user: null };
  }
}

/**
 * Get authenticated admin user from JWT cookies
 */
export async function getAuthUser() {
  try {
    const { verifyJWTFromCookies } = await import('./token');
    const user = await verifyJWTFromCookies();
    
    // Validate admin permissions
    if (user && !user.permissions && user.role !== 'admin') {
      // eslint-disable-next-line no-console
      console.warn('⚠️  Admin: User lacks admin permissions');
      return null;
    }
    
    return user;
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to get auth user:', _error);
    return null;
  }
}

/**
 * Exchange authorization code for tokens (proper OAuth flow)
 * @param code
 * @param codeVerifier
 * @param state
 */
export async function exchangeCodeForTokens(code: string, codeVerifier: string, state: string) {
  try {
    
    // Use consolidated auth config for consistency
    const { authConfig } = await import('../../config/env');
    
    const apiUrl = authConfig.apiUrl;
    const clientId = authConfig.clientId;
    const redirectUri = authConfig.callbackUrl;
    
    const response = await fetch(`${apiUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Accept': 'application/json',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code: code,
        redirect_uri: redirectUri,
        client_id: clientId,
        code_verifier: codeVerifier,
      }),
    })

    if (!response.ok) {
      const errorText = await response.text()
      // eslint-disable-next-line no-console
      console.error('❌ Admin: Token exchange failed:', response.status, response.statusText, errorText)
      throw new Error(`Token exchange failed: ${response.status} ${response.statusText} - ${errorText}`)
    }

    const tokens = await response.json()
    
    return {
      accessToken: tokens.access_token,
      idToken: tokens.id_token,
      refreshToken: tokens.refresh_token,
      expiresIn: tokens.expires_in || 3600, // Include expiry information
    }
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Token exchange error:', _error)
    throw new Error(`Failed to exchange authorization code for tokens: ${_error instanceof Error ? _error.message : 'Unknown error'}`)
  }
}

/**
 * Fetch user info from OAuth userinfo endpoint
 * @param accessToken
 */
export async function getUserInfo(accessToken: string) {
  // Use consolidated auth config for consistency
  const { authConfig } = await import('../../config/env');
  const apiUrl = authConfig.apiUrl;
  
  const response = await fetch(`${apiUrl}/oauth/userinfo`, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    const errorText = await response.text();
    // eslint-disable-next-line no-console
    console.error('❌ Admin: UserInfo fetch failed:', response.status, response.statusText, errorText);
    throw new Error(`UserInfo fetch failed: ${response.status} ${response.statusText} - ${errorText}`);
  }

  return await response.json();
}

/**
 * Redirect to backend Chef Kitchen login with proper PKCE parameters
 * @param callbackUrl
 */
export async function redirectToBackendAdminLogin(callbackUrl?: string): Promise<never> {
  try {
    // Generate proper PKCE parameters
    const { url, codeVerifier, state } = await getAuthorizationUrl();
    
    // Set PKCE cookies for callback
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();
    
    // Store PKCE parameters in cookies for callback
    cookieStore.set('oauth_code_verifier', codeVerifier, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 600, // 10 minutes
      path: '/'
    });
    
    cookieStore.set('oauth_state', state, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 600, // 10 minutes  
      path: '/'
    });
    
    // Store callback URL for after authentication
    if (callbackUrl) {
      cookieStore.set('oauth_redirect_to', callbackUrl, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 600, // 10 minutes
        path: '/'
      });
    }
    
    redirect(url);
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to setup PKCE redirect:', _error);
    // Fallback to simple redirect without PKCE using consolidated config
    const { authConfig } = await import('../../config/env');
    const backendAdminLoginUrl = new URL('/oauth/authorize', authConfig.apiUrl);
    backendAdminLoginUrl.searchParams.set('client_id', authConfig.clientId);
    backendAdminLoginUrl.searchParams.set('redirect_uri', authConfig.callbackUrl);
    backendAdminLoginUrl.searchParams.set('scope', 'openid profile email permissions');
    backendAdminLoginUrl.searchParams.set('response_type', 'code');
    if (callbackUrl) {
      backendAdminLoginUrl.searchParams.set('state', encodeURIComponent(callbackUrl));
    }
    redirect(backendAdminLoginUrl.toString());
  }
}

// ============================================================================
// Additional Auth Helper Functions for Admin Frontend
// ============================================================================

/**
 * Require authentication - redirect to login if not authenticated
 * @param redirectPath
 */
export async function requireAuth(redirectPath?: string) {
  const user = await getAuthUser();
  
  if (!user) {
    const { redirect } = await import('next/navigation');
    const loginUrl = redirectPath ? `/login?redirectTo=${encodeURIComponent(redirectPath)}` : '/login';
    redirect(loginUrl);
  }
  
  return user;
}

/**
 * Clear admin session by removing JWT cookie
 */
export async function clearSession(): Promise<void> {
  try {
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();
    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    cookieStore.delete('access_token');
    cookieStore.delete('id_token');
    cookieStore.delete('refresh_token');
    // Also clear legacy cookie for migration compatibility
    cookieStore.delete('epsx_admin_jwt');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to clear session:', _error);
    throw _error;
  }
}

/**
 * Check if user has specific admin permission
 * @param permission
 */
export async function hasAdminPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) {return false;}
    
    // Admin users have broader permissions
    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    return permissions.includes(permission) || 
           permissions.includes('admin:*') ||
           permissions.includes('*');
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to check permission:', _error);
    return false;
  }
}

/**
 * Check if user has specific permission
 * @param permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) {return false;}
    
    // Admin users have broader permissions
    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    return permissions.includes(permission) || 
           permissions.includes('admin:*:*') ||
           permissions.some(p => p.startsWith('admin:'));
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Admin: Failed to check permission:', _error);
    return false;
  }
}

/**
 * Require specific structured permission
 * @param permission
 * @param redirectPath
 */
export async function requirePermission(permission: string, redirectPath?: string) {
  const { redirect } = await import('next/navigation');
  const user = await requireAuth(redirectPath);
  
  const hasRequiredPermission = await hasPermission(permission);
  
  if (!hasRequiredPermission) {
    const accessDeniedUrl = `/access-denied?permission=${encodeURIComponent(permission)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}