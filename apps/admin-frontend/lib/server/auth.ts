/**
 * Server-side OAuth utilities for admin-frontend
 */

import { redirect } from 'next/navigation';

/**
 * Generate PKCE code verifier
 */
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Generate PKCE code challenge
 */
async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(digest));
}

/**
 * Generate random string
 */
function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Base64 URL encode
 */
function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Generate authorization URL with PKCE parameters
 */
export async function getAuthorizationUrl() {
  // Use consolidated auth config
  const { authConfig } = await import('../../config/env');
  
  console.log('🔄 Admin: Generating PKCE parameters for OAuth authorization...');
  
  // Generate PKCE parameters (server-side only)
  const codeVerifier = generateCodeVerifier();
  console.log('✅ Admin: Code verifier generated successfully');
  
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  console.log('✅ Admin: Code challenge generated successfully');
  
  const state = generateRandomString(32);
  console.log('✅ Admin: State parameter generated successfully');
  
  // Build authorization URL using consolidated config
  const authorizationEndpoint = authConfig.authorizationEndpoint;
  const clientId = authConfig.clientId;
  const redirectUri = authConfig.callbackUrl;
  
  console.log('🔧 Admin: OAuth configuration:', {
    authorizationEndpoint,
    clientId,
    redirectUri,
    scope: 'openid profile email permissions'
  });
  
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'openid profile email permissions',
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });
  
  const url = `${authorizationEndpoint}?${params.toString()}`;
  console.log('✅ Admin: Authorization URL generated successfully:', url);
  
  return {
    url,
    codeVerifier,
    state,
  };
}

/**
 * Get server session with JWT verification
 */
export async function getServerSession() {
  try {
    const { getSessionFromJWT } = await import('./jwt');
    const session = await getSessionFromJWT();
    return session;
  } catch (error) {
    console.error('❌ Admin: Failed to get server session:', error);
    return { isAuthenticated: false, user: null };
  }
}

/**
 * Get authenticated admin user from JWT cookies
 */
export async function getAuthUser() {
  try {
    const { verifyJWTFromCookies } = await import('./jwt');
    const user = await verifyJWTFromCookies();
    
    // Validate admin permissions
    if (user && !user.permissions && user.role !== 'admin') {
      console.warn('⚠️  Admin: User lacks admin permissions');
      return null;
    }
    
    return user;
  } catch (error) {
    console.error('❌ Admin: Failed to get auth user:', error);
    return null;
  }
}

/**
 * Exchange authorization code for tokens (proper OAuth flow)
 */
export async function exchangeCodeForTokens(code: string, codeVerifier: string, state: string) {
  try {
    console.log('🔄 Admin: Exchanging authorization code for access token...')
    
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
    console.log('✅ Admin: Successfully received tokens from backend')
    
    return {
      accessToken: tokens.access_token,
      idToken: tokens.id_token,
      refreshToken: tokens.refresh_token,
      expiresIn: tokens.expires_in || 3600, // Include expiry information
    }
  } catch (error) {
    console.error('❌ Admin: Token exchange error:', error)
    throw new Error(`Failed to exchange authorization code for tokens: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Fetch user info from OAuth userinfo endpoint
 */
export async function getUserInfo(accessToken: string) {
  // Use consolidated auth config for consistency
  const { authConfig } = await import('../../config/env');
  const apiUrl = authConfig.apiUrl;
  
  console.log('🔄 Admin: Fetching user info from backend userinfo endpoint:', apiUrl);
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

  const userinfo = await response.json();
  console.log('✅ Admin: Successfully received user info');
  return userinfo;
}

/**
 * Redirect to backend Chef Kitchen login with proper PKCE parameters
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
    
    console.log('✅ Admin: PKCE parameters stored in cookies, redirecting to OAuth');
    redirect(url);
  } catch (error) {
    console.error('❌ Admin: Failed to setup PKCE redirect:', error);
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
    console.log('✅ Admin: OIDC session cleared successfully');
  } catch (error) {
    console.error('❌ Admin: Failed to clear session:', error);
    throw error;
  }
}

/**
 * Check if user has specific admin permission
 */
export async function hasAdminPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Admin users have broader permissions
    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    return permissions.includes(permission) || 
           permissions.includes('admin:*') ||
           permissions.includes('*');
  } catch (error) {
    console.error('❌ Admin: Failed to check permission:', error);
    return false;
  }
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Admin users have broader permissions
    const permissions = Array.isArray(user.permissions) ? user.permissions : [];
    return permissions.includes(permission) || 
           permissions.includes('admin:*:*') ||
           permissions.some(p => p.startsWith('admin:'));
  } catch (error) {
    console.error('❌ Admin: Failed to check permission:', error);
    return false;
  }
}

/**
 * Require specific structured permission
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