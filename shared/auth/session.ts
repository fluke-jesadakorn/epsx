/**
 * Shared Session Management Utilities
 * Consolidates OIDC session handling across applications
 */

import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { OIDC_COOKIES, setOIDCTokens, clearOIDCTokens, clearLegacyCookies } from './cookies';
import { getBackendUrl, URL, URLContext, OIDCEndpoint } from '../utils/url-resolver';

export interface SessionConfig {
  appType: 'frontend' | 'admin';
  requireAdminPermissions?: boolean;
  legacyJwtCookie?: string;
}

export interface SessionUser {
  id: string;
  email: string;
  name: string;
  permissions: string[];
  platform_context?: any;
}

export interface SessionResponse {
  isAuthenticated: boolean;
  user: SessionUser | null;
  expiresAt?: number;
  tokenType?: string;
  adminAccess?: boolean;
  error?: string;
}

export async function getSession(config: SessionConfig): Promise<NextResponse> {
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  try {
    const cookieStore = await cookies();
    
    // Get OIDC tokens from cookies
    const accessToken = cookieStore.get(OIDC_COOKIES.ACCESS_TOKEN)?.value;
    const idToken = cookieStore.get(OIDC_COOKIES.ID_TOKEN)?.value;
    const refreshToken = cookieStore.get(OIDC_COOKIES.REFRESH_TOKEN)?.value;

    console.log(`🔍 ${appLabel} OIDC Cookie Check:`, {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing', 
      refreshToken: refreshToken ? 'present' : 'missing'
    });
    
    // Return unauthenticated response if no tokens (this is normal for public access)
    if (!accessToken || !idToken) {
      console.log(`ℹ️ No OIDC tokens found for ${appLabel.toLowerCase()} session - returning unauthenticated state`);
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        tokenType: 'none'
      } as SessionResponse, { status: 200 });
    }

    // Get user info from backend
    const userInfoEndpoint = config.appType === 'admin' 
      ? URL.oidc(OIDCEndpoint.USERINFO, URLContext.SERVER)
      : `${getBackendUrl('server')}/oauth/userinfo`;
      
    const userInfoResponse = await fetch(userInfoEndpoint, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    });

    if (!userInfoResponse.ok) {
      console.error(`❌ Failed to fetch ${appLabel.toLowerCase()} user info:`, userInfoResponse.status);
      
      // If access token expired, suggest refresh
      if (userInfoResponse.status === 401 && refreshToken) {
        console.log(`🔄 ${appLabel} access token expired, attempting refresh...`);
        return NextResponse.json({
          isAuthenticated: false,
          user: null,
          error: `${appLabel} access token expired, refresh needed`
        } as SessionResponse, { status: 401 });
      }
      
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: `Failed to validate ${appLabel.toLowerCase()} session with backend`
      } as SessionResponse, { status: 401 });
    }

    const userInfo = await userInfoResponse.json();
    
    // Validate admin permissions if required
    if (config.requireAdminPermissions) {
      const hasAdminAccess = userInfo.permissions?.some((p: string) => 
        p === 'admin:*:*' || p.startsWith('admin:')
      ) || false;
      
      if (!hasAdminAccess) {
        console.warn('⚠️ User lacks admin permissions for admin frontend', {
          permissions: userInfo.permissions,
          required: 'admin:*:* or admin:{resource}:{action}'
        });
        return NextResponse.json({
          isAuthenticated: false,
          user: null,
          error: 'Insufficient admin permissions'
        } as SessionResponse, { status: 403 });
      }
      
      console.log(`✅ Successfully retrieved ${appLabel.toLowerCase()} user info from OIDC backend`);
    }

    // Return session data
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        id: userInfo.sub,
        email: userInfo.email,
        name: userInfo.name,
        permissions: userInfo.permissions || [],
        platform_context: userInfo.platform_context,
      },
      expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour from now
      tokenType: 'oidc',
      ...(config.requireAdminPermissions && { adminAccess: true })
    } as SessionResponse);

  } catch (error) {
    console.error(`❌ ${appLabel} OIDC Session API error:`, error);
    
    return NextResponse.json({
      isAuthenticated: false,
      user: null,
      error: `OIDC ${appLabel.toLowerCase()} session verification failed`
    } as SessionResponse, { status: 500 });
  }
}

export async function storeSession(
  request: NextRequest,
  config: SessionConfig
): Promise<NextResponse> {
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  try {
    const body = await request.json();
    const { accessToken, idToken, refreshToken } = body;
    
    // Validate required OIDC tokens
    if (!accessToken || !idToken || !refreshToken) {
      return NextResponse.json({
        error: `All OIDC tokens (accessToken, idToken, refreshToken) required for ${appLabel.toLowerCase()} access`
      }, { status: 400 });
    }
    
    // Validate admin permissions if required
    if (config.requireAdminPermissions) {
      const userInfoEndpoint = URL.oidc(OIDCEndpoint.USERINFO, URLContext.SERVER);
      const userInfoResponse = await fetch(userInfoEndpoint, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json'
        }
      });

      if (!userInfoResponse.ok) {
        return NextResponse.json({
          error: 'Invalid access token provided'
        }, { status: 401 });
      }

      const userInfo = await userInfoResponse.json();
      
      const hasAdminAccess = userInfo.permissions?.some((p: string) => 
        p === 'admin:*:*' || p.startsWith('admin:')
      ) || false;
      
      if (!hasAdminAccess) {
        return NextResponse.json({
          error: 'Insufficient admin permissions - admin:* required'
        }, { status: 403 });
      }
    }
    
    const response = NextResponse.json({ 
      success: true, 
      tokenType: 'oidc',
      ...(config.requireAdminPermissions && { adminAccess: true })
    });
    
    // Store OIDC tokens
    setOIDCTokens(response, { accessToken, idToken, refreshToken });
    
    // Clean up legacy cookies
    if (config.legacyJwtCookie) {
      response.cookies.delete(config.legacyJwtCookie);
    }
    
    console.log(`✅ ${appLabel} OIDC tokens stored successfully`);
    
    return response;
    
  } catch (error) {
    console.error(`❌ ${appLabel} OIDC Session store error:`, error);
    return NextResponse.json({
      error: `Failed to store ${appLabel.toLowerCase()} OIDC session`
    }, { status: 500 });
  }
}

export async function clearSession(config: SessionConfig): Promise<NextResponse> {
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  try {
    const response = NextResponse.json({ 
      success: true, 
      tokenType: 'oidc',
      ...(config.requireAdminPermissions && { adminAccess: false })
    });
    
    // Clear OIDC cookies
    clearOIDCTokens(response);
    clearLegacyCookies(response);
    
    console.log(`✅ All ${appLabel.toLowerCase()} OIDC session cookies cleared`);
    
    return response;
    
  } catch (error) {
    console.error(`❌ ${appLabel} OIDC Session clear error:`, error);
    return NextResponse.json({
      error: `Failed to clear ${appLabel.toLowerCase()} OIDC session`
    }, { status: 500 });
  }
}

export async function refreshSession(config: SessionConfig): Promise<NextResponse> {
  const appLabel = config.appType === 'admin' ? 'Admin' : 'Frontend';
  
  try {
    const cookieStore = await cookies();
    
    // Get refresh token
    const refreshToken = cookieStore.get(OIDC_COOKIES.REFRESH_TOKEN)?.value;
    
    if (!refreshToken) {
      return NextResponse.json({
        error: `No valid refresh token for ${appLabel.toLowerCase()} session refresh`
      }, { status: 401 });
    }
    
    // Use OIDC token refresh endpoint
    const backendUrl = getBackendUrl('server');
    
    console.log(`🔄 Refreshing ${appLabel.toLowerCase()} OIDC tokens using refresh token...`);
    
    const response = await fetch(`${backendUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded'
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: config.appType === 'admin' ? 'epsx-admin' : 'epsx-frontend'
      })
    });
    
    if (!response.ok) {
      console.error(`❌ ${appLabel} OIDC token refresh failed:`, response.status);
      return NextResponse.json({
        error: `${appLabel} refresh token expired or invalid`
      }, { status: 401 });
    }
    
    const tokens = await response.json();
    console.log(`✅ Successfully refreshed ${appLabel.toLowerCase()} OIDC tokens`);
    
    // Validate admin permissions if required
    if (config.requireAdminPermissions) {
      const userInfoEndpoint = URL.oidc(OIDCEndpoint.USERINFO, URLContext.SERVER);
      const userInfoResponse = await fetch(userInfoEndpoint, {
        headers: {
          'Authorization': `Bearer ${tokens.access_token}`,
          'Content-Type': 'application/json'
        }
      });
      
      if (userInfoResponse.ok) {
        const userInfo = await userInfoResponse.json();
        
        const hasAdminAccess = userInfo.permissions?.some((p: string) => 
          p === 'admin:*:*' || p.startsWith('admin:')
        ) || false;
        
        if (!hasAdminAccess) {
          return NextResponse.json({
            error: 'Admin privileges revoked'
          }, { status: 403 });
        }
      }
    }
    
    // Update cookies with new tokens
    const refreshResponse = NextResponse.json({
      message: `${appLabel} OIDC tokens refreshed successfully`,
      expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour from now
      tokenType: 'oidc',
      ...(config.requireAdminPermissions && { adminAccess: true })
    });
    
    // Set new tokens
    setOIDCTokens(refreshResponse, {
      accessToken: tokens.access_token,
      idToken: tokens.id_token,
      refreshToken: tokens.refresh_token || refreshToken,
      expiresIn: tokens.expires_in
    });
    
    return refreshResponse;
    
  } catch (error) {
    console.error(`❌ ${appLabel} OIDC Session refresh error:`, error);
    return NextResponse.json({
      error: `Failed to refresh ${appLabel.toLowerCase()} OIDC session`
    }, { status: 500 });
  }
}