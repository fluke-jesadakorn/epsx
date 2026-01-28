/**
 * Server-side OAuth utilities for admin-frontend
 */

import { redirect } from 'next/navigation';

import { generateCodeChallenge, generateCodeVerifier, generateRandomString } from '@/shared/auth/pkce';

// OAuth authorization URL generation now handled by shared utilities
async function getAuthorizationUrl(): Promise<{ url: string; codeVerifier: string; state: string }> {
  const codeVerifier = generateCodeVerifier();
  // codeChallenge generated but unused in current simplified flow
  await generateCodeChallenge(codeVerifier);
  const state = generateRandomString(32);
  return { url: '/auth', codeVerifier, state };
}

/**
 * Get server session with JWT verification
 */
export async function getServerSession(): Promise<{ isAuthenticated: boolean; user: any | null }> {
  try {
    const { getSessionFromJWT } = await import('./token');
    return await getSessionFromJWT();
  } catch (_error) {

    console.error('❌ Admin: Failed to get server session:', _error);
    return { isAuthenticated: false, user: null };
  }
}

/**
 * Get authenticated admin user from JWT cookies
 */
export async function getAuthUser(): Promise<any | null> {
  try {
    const { verifyJWTFromCookies } = await import('./token');
    const user = await verifyJWTFromCookies();

    // PERMISSION REFACTOR: Backend (Rust) enforces actual admin access.
    // If a valid JWT is present, we consider the user authenticated.
    return user;

    return user;
  } catch (_error) {

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
export async function exchangeCodeForTokens(
  code: string,
  codeVerifier: string,
  state: string
): Promise<{ accessToken: string; idToken: string; refreshToken: string; expiresIn: number }> {
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

    console.error('❌ Admin: Token exchange error:', _error)
    throw new Error(`Failed to exchange authorization code for tokens: ${_error instanceof Error ? _error.message : 'Unknown error'}`)
  }
}

/**
 * Fetch user info from OAuth userinfo endpoint
 * @param accessToken
 */
export async function getUserInfo(accessToken: string): Promise<any> {
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
export async function requireAuth(redirectPath?: string): Promise<any> {
  const user = await getAuthUser();

  if (!user) {
    const { redirect } = await import('next/navigation');
    const loginUrl = redirectPath ? `/auth?return_url=${encodeURIComponent(redirectPath)}` : '/auth';
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
    const { COOKIES } = await import('@/shared/auth/cookies');
    const cookieStore = await cookies();
    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    cookieStore.delete(COOKIES.access_token);
    cookieStore.delete(COOKIES.id_token);
    cookieStore.delete(COOKIES.refresh_token);
    // Also clear legacy cookie for migration compatibility
    cookieStore.delete('epsx_admin_jwt');
  } catch (_error) {

    console.error('❌ Admin: Failed to clear session:', _error);
    throw _error;
  }
}

// Permission check stubs - Backend handles all enforcement via JWT middleware
/**
 *
 * @param _permission
 */
export async function hasAdminPermission(_permission: string): Promise<boolean> {
  return true;
}

/**
 *
 * @param _permission
 */
export async function hasPermission(_permission: string): Promise<boolean> {
  return true;
}

export async function requirePermission(_permission: string, redirectPath?: string): Promise<any> {
  // PERMISSION REFACTOR: Admin-frontend is permissive; backend enforces access.
  return await requireAuth(redirectPath);
}