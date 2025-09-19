/**
 * Server-side OIDC Cookie Utilities for Frontend
 * OIDC Migration: Uses OIDC-compliant cookies instead of custom JWT
 */
import { cookies } from 'next/headers';
import { verifyJWT, type EPSXJWTPayload } from '../../../../shared/auth/jwt';
import { URL, URLContext, OIDCEndpoint } from '../../../../shared/utils/url-resolver';

/**
 * OIDC Migration: Get access token from OIDC cookies
 */
export async function getOIDCAccessTokenFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('access_token')?.value || null;
  } catch (error) {
    console.error('❌ Failed to get OIDC access token from cookies:', error);
    return null;
  }
}

/**
 * OIDC Migration: Get ID token from OIDC cookies
 */
export async function getOIDCIdTokenFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('id_token')?.value || null;
  } catch (error) {
    console.error('❌ Failed to get OIDC ID token from cookies:', error);
    return null;
  }
}

/**
 * OIDC Migration: Get refresh token from OIDC cookies
 */
export async function getOIDCRefreshTokenFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    return cookieStore.get('refresh_token')?.value || null;
  } catch (error) {
    console.error('❌ Failed to get OIDC refresh token from cookies:', error);
    return null;
  }
}

/**
 * OIDC Migration: Get user info from backend using OIDC access token
 */
export async function getUserInfoFromOIDC(): Promise<EPSXJWTPayload | null> {
  try {
    const accessToken = await getOIDCAccessTokenFromCookies();
    if (!accessToken) {
      return null;
    }
    
    // Get user info from backend OIDC userinfo endpoint
    const userinfoEndpoint = URL.oidc(OIDCEndpoint.USERINFO, URLContext.SERVER);
    const response = await fetch(userinfoEndpoint, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      console.error('❌ Failed to get user info from OIDC backend:', response.status);
      return null;
    }

    const userInfo = await response.json();
    
    // Convert to EPSXJWTPayload format for compatibility
    return {
      sub: userInfo.sub,
      email: userInfo.email,
      name: userInfo.name,
      permissions: userInfo.permissions || [],
      platform_context: userInfo.platform_context,
      exp: Math.floor(Date.now() / 1000) + (60 * 60), // 1 hour from now
      iat: Math.floor(Date.now() / 1000),
      iss: 'epsx-backend-oidc',
      aud: 'epsx-frontend',
    } as EPSXJWTPayload;
    
  } catch (error) {
    console.error('❌ Failed to get user info from OIDC:', error);
    return null;
  }
}

/**
 * OIDC Migration: Get user session data from OIDC cookies and backend
 */
export async function getSessionFromOIDC(): Promise<{
  isAuthenticated: boolean;
  user: EPSXJWTPayload | null;
}> {
  try {
    const userInfo = await getUserInfoFromOIDC();
    
    if (!userInfo) {
      return { isAuthenticated: false, user: null };
    }
    
    return { isAuthenticated: true, user: userInfo };
  } catch (error) {
    console.error('❌ Failed to get session from OIDC:', error);
    return { isAuthenticated: false, user: null };
  }
}

// ============================================================================
// Legacy JWT Support (Backward Compatibility)
// ============================================================================

/**
 * Legacy: Get JWT token from httpOnly cookies (for backward compatibility)
 * @deprecated Use getOIDCAccessTokenFromCookies instead
 */
export async function getJWTFromCookies(): Promise<string | null> {
  try {
    const cookieStore = await cookies();
    // Check both legacy JWT cookie and new access token
    return cookieStore.get('epsx_frontend_jwt')?.value || 
           cookieStore.get('access_token')?.value ||
           null;
  } catch (error) {
    console.error('❌ Failed to get JWT from cookies:', error);
    return null;
  }
}

/**
 * Legacy: Verify and decode JWT token from cookies (for backward compatibility)
 * @deprecated Use getUserInfoFromOIDC instead
 */
export async function verifyJWTFromCookies(): Promise<EPSXJWTPayload | null> {
  try {
    // First try OIDC method
    const oidcUserInfo = await getUserInfoFromOIDC();
    if (oidcUserInfo) {
      return oidcUserInfo;
    }
    
    // Fallback to legacy JWT
    const token = await getJWTFromCookies();
    if (!token) return null;
    
    return await verifyJWT(token);
  } catch (error) {
    console.error('❌ Failed to verify JWT from cookies:', error);
    return null;
  }
}

/**
 * Legacy: Get user session data from JWT cookies (for backward compatibility)
 * @deprecated Use getSessionFromOIDC instead
 */
export async function getSessionFromJWT(): Promise<{
  isAuthenticated: boolean;
  user: EPSXJWTPayload | null;
}> {
  try {
    // Use OIDC method first
    return await getSessionFromOIDC();
  } catch (error) {
    console.error('❌ Failed to get session from JWT:', error);
    return { isAuthenticated: false, user: null };
  }
}