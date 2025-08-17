/**
 * Enhanced Server-Side Authentication for Frontend
 * Uses JWT-based authentication with jose library and OAuth 2.0 flow
 */
import { redirect } from 'next/navigation';
import { verifyJWTFromCookies, getSessionFromJWT } from './jwt';
import { hasPermissionInJWT, hasPackageTierInJWT, type EPSXJWTPayload } from '@epsx/auth-shared';

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
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await getAuthUser();
  
  if (!user) {
    const loginUrl = `/login${redirectPath ? `?callbackUrl=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(loginUrl);
  }
  
  return user;
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    return user.permissions.includes(permission) || user.permissions.includes('*');
  } catch (error) {
    console.error('❌ Failed to check permission:', error);
    return false;
  }
}

/**
 * Require specific permission - redirect to access denied if not found
 */
export async function requirePermission(permission: string, redirectPath?: string): Promise<EPSXJWTPayload> {
  const user = await requireAuth(redirectPath);
  
  const hasRequiredPermission = user.permissions.includes(permission) || user.permissions.includes('*');
  
  if (!hasRequiredPermission) {
    const accessDeniedUrl = `/access-denied?permission=${encodeURIComponent(permission)}${redirectPath ? `&route=${encodeURIComponent(redirectPath)}` : ''}`;
    redirect(accessDeniedUrl);
  }
  
  return user;
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
    
    const userLevel = tierHierarchy[user.package_tier] || 0;
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
 * Check if user has specific role
 */
export async function hasRole(requiredRole: string): Promise<boolean> {
  try {
    const user = await getAuthUser();
    if (!user) return false;
    
    const roleHierarchy: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };
    
    const userLevel = roleHierarchy[user.role.toLowerCase()] || 0;
    const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1;
    
    return userLevel >= requiredLevel;
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
    const authorizationEndpoint = `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/oauth/authorize`
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
    const redirectUri = `${process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'}/api/auth/callback/epsx-backend`
    
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
    
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'
    const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
    const redirectUri = `${process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'}/api/auth/callback/epsx-backend`
    
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
  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'
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
 * Generate code verifier for PKCE using Node.js crypto
 */
function generateCodeVerifier(): string {
  // Server-side: use Node.js crypto
  const crypto = require('crypto')
  return crypto.randomBytes(32).toString('base64url')
}

/**
 * Generate code challenge from verifier using SHA-256
 */
async function generateCodeChallenge(verifier: string): Promise<string> {
  // Server-side: use Node.js crypto
  const crypto = require('crypto')
  return crypto.createHash('sha256').update(verifier).digest('base64url')
}

/**
 * Generate cryptographically secure random string
 */
function generateRandomString(length: number): string {
  // Server-side: use Node.js crypto
  const crypto = require('crypto')
  return crypto.randomBytes(length).toString('base64url')
}