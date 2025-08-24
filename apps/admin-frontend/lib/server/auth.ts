/**
 * Server-side OAuth utilities for admin-frontend
 */

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
    scope: 'openid profile email admin_modules'
  });
  
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'openid profile email admin_modules',
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
    if (user && !user.admin_modules && user.role !== 'admin' && user.role !== 'super_admin') {
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
    
    // Use internal Docker network URL for server-side requests
    const apiUrl = process.env.NODE_ENV === 'production' 
      ? (process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io')
      : 'http://localhost:8080'
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-admin'
    const redirectUri = `${process.env.NEXT_PUBLIC_ADMIN_URL || 'https://admin.epsx.io'}/api/auth/callback/epsx-backend`
    
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
  // Use internal Docker network URL for server-side requests
  const apiUrl = process.env.NODE_ENV === 'production' 
    ? (process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io')
    : 'http://localhost:8080';
  
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
 * Redirect to backend Chef Kitchen login with callback URL
 */
export function redirectToBackendAdminLogin(callbackUrl?: string): never {
  const backendAdminLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
  backendAdminLoginUrl.searchParams.set('client_id', 'epsx-admin'); // Admin client ID for Chef Kitchen theme
  backendAdminLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_ADMIN_URL || 'https://admin.epsx.io'}/api/auth/callback/epsx-backend`);
  backendAdminLoginUrl.searchParams.set('scope', 'openid profile email admin_modules');
  backendAdminLoginUrl.searchParams.set('response_type', 'code');
  if (callbackUrl) {
    backendAdminLoginUrl.searchParams.set('state', encodeURIComponent(callbackUrl));
  }
  redirect(backendAdminLoginUrl.toString());
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
    redirectToBackendAdminLogin(redirectPath);
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
    cookieStore.delete('epsx_admin_jwt');
    console.log('✅ Admin: User session cleared successfully');
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
    return user.permissions.includes(permission) || 
           user.permissions.includes('admin:*') ||
           user.permissions.includes('*');
  } catch (error) {
    console.error('❌ Admin: Failed to check permission:', error);
    return false;
  }
}

/**
 * Check if user has access to specific admin module
 */
export async function hasAdminModule(module: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Check admin_modules array
    if (user.admin_modules && user.admin_modules.includes(module)) {
      return true;
    }
    
    // Super admin has access to all modules
    if (user.role === 'super_admin') {
      return true;
    }
    
    return false;
  } catch (error) {
    console.error('❌ Admin: Failed to check module access:', error);
    return false;
  }
}

/**
 * Require specific admin module access
 */
export async function requireAdminModule(module: string, redirectPath?: string) {
  const { redirect } = await import('next/navigation');
  const user = await requireAuth(redirectPath);
  
  const hasModule = await hasAdminModule(module);
  
  if (!hasModule) {
    const accessDeniedUrl = `/access-denied?module=${encodeURIComponent(module)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}