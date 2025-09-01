/**
 * Enhanced Server-Side Authentication for Frontend
 * Uses JWT-based authentication with jose library and OAuth 2.0 flow
 */
import { redirect } from 'next/navigation';
import { cookies } from 'next/headers';
import { verifyJWTFromCookies, getSessionFromJWT } from './jwt';
import { type EPSXJWTPayload, derivePackageTierFromPermissions, hasJWTPermission, isJWTAdmin } from '@/lib/auth-utils';

/**
 * Get authenticated user from JWT cookies
 */
export async function getAuthUser(): Promise<EPSXJWTPayload | null> {
  try {
    return await verifyJWTFromCookies();
  } catch (error) {
    console.error('❌ Failed to get authenticated user:', error);
    return null;
  }
}

/**
 * Clear user session by removing JWT cookie
 */
export async function clearSession(): Promise<void> {
  try {
    const cookieStore = await cookies();
    cookieStore.delete('epsx_frontend_jwt');
    console.log('✅ Frontend: User session cleared successfully');
  } catch (error) {
    console.error('❌ Failed to clear session:', error);
    throw error;
  }
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await getAuthUser();
  
  if (!user) {
    redirectToBackendLogin(redirectPath);
  }
  
  return user;
}

/**
 * Check if user has specific feature access using simple role system
 */
export async function hasFeatureAccess(feature: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Map feature to structured permission
    const permission = mapFeatureToPermission(feature);
    return hasJWTPermission(user, permission);
  } catch (error) {
    console.error('❌ Failed to check feature access:', error);
    return false;
  }
}

/**
 * Require specific feature access - redirect to access denied if not found
 */
export async function requireFeatureAccess(feature: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasAccess = await hasFeatureAccess(feature);
  
  if (!hasAccess) {
    const accessDeniedUrl = `/access-denied?feature=${encodeURIComponent(feature)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}

/**
 * Legacy permission check - maps old permission strings to features
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Map legacy permissions to structured permissions
    const structuredPermission = mapLegacyPermission(permission);
    return hasJWTPermission(user, structuredPermission);
  } catch (error) {
    console.error('❌ Failed to check permission:', error);
    return false;
  }
}

/**
 * Legacy permission requirement - maps to feature access
 */
export async function requirePermission(permission: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  // Map legacy permissions to features and redirect accordingly
  switch (permission) {
    case 'users.view':
    case 'dashboard.view':
    case 'analytics.view':
      return requireFeatureAccess('view_eps', redirectPath);
    
    case 'analytics.export':
      return requireFeatureAccess('export_data', redirectPath);
    
    case 'admin':
    case 'admin.users':
      return requireRole('admin', redirectPath);
    
    default:
      return requireFeatureAccess(permission, redirectPath);
  }
}

/**
 * Check if user has specific package tier or higher
 */
export async function hasPackageTier(requiredTier: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    const tierHierarchy: Record<string, number> = {
      'FREE': 1,
      'BRONZE': 2,
      'SILVER': 3,
      'GOLD': 4,
      'PLATINUM': 5,
      'ENTERPRISE': 6
    };
    
    const userPackageTier = derivePackageTierFromPermissions(user.permissions);
    const userLevel = tierHierarchy[userPackageTier] || 0;
    const requiredLevel = tierHierarchy[requiredTier] || 1;
    
    return userLevel >= requiredLevel;
  } catch (error) {
    console.error('❌ Failed to check package tier:', error);
    return false;
  }
}

/**
 * Require specific package tier - redirect to upgrade if not found
 */
export async function requirePackageTier(requiredTier: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredTier = await hasPackageTier(requiredTier);
  
  if (!hasRequiredTier) {
    const upgradeUrl = `/payment?tier=${encodeURIComponent(requiredTier)}${redirectPath ? `&callbackUrl=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(upgradeUrl);
  }
  
  return user;
}

/**
 * Check if user has specific role using simple role system
 */
export async function hasRole(requiredRole: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    // Map role to permission check
    switch (requiredRole.toLowerCase()) {
      case 'admin':
        return isJWTAdmin(user);
      case 'user':
        return hasJWTPermission(user, 'epsx:analytics:view');
      case 'guest':
        return hasJWTPermission(user, 'epsx:analytics:view');
      default:
        return false;
    }
  } catch (error) {
    console.error('❌ Failed to check role:', error);
    return false;
  }
}

/**
 * Require specific role - redirect to access denied if not found
 */
export async function requireRole(requiredRole: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredRole = await hasRole(requiredRole);
  
  if (!hasRequiredRole) {
    const accessDeniedUrl = `/access-denied?role=${encodeURIComponent(requiredRole)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
}

/**
 * Redirect to backend Pancake login with callback URL
 */
export function redirectToBackendLogin(callbackUrl?: string): never {
  const backendLoginUrl = new URL('/oauth/authorize', process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io');
  backendLoginUrl.searchParams.set('client_id', process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend');
  backendLoginUrl.searchParams.set('redirect_uri', `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`);
  backendLoginUrl.searchParams.set('scope', 'openid profile email');
  backendLoginUrl.searchParams.set('response_type', 'code');
  if (callbackUrl) {
    backendLoginUrl.searchParams.set('state', encodeURIComponent(callbackUrl));
  }
  redirect(backendLoginUrl.toString());
}

// ============================================================================
// OAuth 2.0 / OIDC Server-Side Functions
// ============================================================================

/**
 * Generate OAuth authorization URL with PKCE for server-side use
 * This function runs on the server and generates secure PKCE parameters
 */
export async function getAuthorizationUrl() {
  try {
    console.log('🔄 Frontend: Generating PKCE parameters for OAuth authorization...')
    
    // Generate PKCE parameters (server-side only)
    const codeVerifier = generateCodeVerifier()
    console.log('✅ Frontend: Code verifier generated successfully')
    
    const codeChallenge = await generateCodeChallenge(codeVerifier)
    console.log('✅ Frontend: Code challenge generated successfully')
    
    const state = generateRandomString(32)
    console.log('✅ Frontend: State parameter generated successfully')
    
    // Build authorization URL
    const authorizationEndpoint = `${process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io'}/oauth/authorize`
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
    const redirectUri = `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`
    
    console.log('🔧 Frontend: OAuth configuration:', {
      authorizationEndpoint,
      clientId,
      redirectUri,
      scope: 'openid profile email'
    })
    
    const params = new URLSearchParams({
      response_type: 'code',
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: 'openid profile email',
      state: state,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    })
    
    const url = `${authorizationEndpoint}?${params.toString()}`
    console.log('✅ Frontend: Authorization URL generated successfully:', url)
    
    return {
      url,
      codeVerifier,
      state,
    }
  } catch (error) {
    console.error('❌ Frontend: Failed to generate authorization URL:', error)
    throw new Error(`OAuth authorization URL generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Exchange authorization code for tokens (simplified flow)
 */
export async function exchangeCodeForTokens(code: string, codeVerifier: string, state: string) {
  try {
    console.log('🔄 Frontend: Exchanging authorization code for access token...')
    
    // Use internal Docker network URL for server-side requests
    const apiUrl = process.env.NODE_ENV === 'production' 
      ? (process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io')
      : (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080')
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
    const redirectUri = `${process.env.NEXT_PUBLIC_APP_URL || 'https://epsx.io'}/api/auth/callback/epsx-backend`
    
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
      console.error('❌ Frontend: Token exchange failed:', response.status, response.statusText, errorText)
      throw new Error(`Token exchange failed: ${response.status} ${response.statusText} - ${errorText}`)
    }

    const tokens = await response.json()
    console.log('✅ Frontend: Successfully received tokens from backend')
    
    return {
      accessToken: tokens.access_token,
      idToken: tokens.id_token,
      refreshToken: tokens.refresh_token,
    }
  } catch (error) {
    console.error('❌ Frontend: Token exchange error:', error)
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
    : (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080')
  const response = await fetch(`${apiUrl}/oauth/userinfo`, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  })

  if (!response.ok) {
    throw new Error(`UserInfo fetch failed: ${response.status} ${response.statusText}`)
  }

  return await response.json()
}

// ============================================================================
// PKCE Helper Functions (Server-Side Only)
// ============================================================================

/**
 * Generate code verifier for PKCE using Web Crypto API (Edge Runtime compatible)
 */
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Generate code challenge from verifier using SHA-256
 */
async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(digest));
}

/**
 * Generate cryptographically secure random string
 */
function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Base64 URL encode (Edge Runtime compatible)
 */
function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Map feature to structured permission
 */
function mapFeatureToPermission(feature: string): string {
  const featureMap: Record<string, string> = {
    'view_eps': 'epsx:analytics:view',
    'export_data': 'epsx:analytics:export',
    'advanced_analytics': 'epsx:analytics:advanced',
    'realtime_data': 'epsx:realtime:access',
    'manage_profile': 'epsx:profile:manage',
    'admin_users': 'admin:users:manage',
    'admin_system': 'admin:system:manage',
  };
  
  return featureMap[feature] || `epsx:${feature}:access`;
}

/**
 * Map legacy permission to structured permission
 */
function mapLegacyPermission(permission: string): string {
  const permissionMap: Record<string, string> = {
    'users.view': 'epsx:users:read',
    'dashboard.view': 'epsx:analytics:view',
    'analytics.view': 'epsx:analytics:view',
    'analytics.export': 'epsx:analytics:export',
    'admin': 'admin:*:*',
    'admin.users': 'admin:users:manage',
  };
  
  return permissionMap[permission] || permission;
}