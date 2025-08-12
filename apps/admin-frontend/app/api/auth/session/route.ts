import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

interface SessionData {
  userId: string;
  email: string;
  name?: string;
  picture?: string;
  role: string;
  permissions: string[];
  emailVerified: boolean;
  sessionId: string;
  issuedAt: number;
  expiresAt: number;
}

/**
 * Server-side session validation endpoint
 * Returns current user session information
 */
export async function GET(request: NextRequest) {
  const cookieStore = await cookies();
  
  try {
    // Get session cookie
    const sessionCookie = cookieStore.get('admin_session');
    const accessToken = cookieStore.get('admin_access_token');
    
    if (!sessionCookie?.value) {
      return NextResponse.json(
        { authenticated: false, error: 'No session found' },
        { status: 401 }
      );
    }
    
    // Parse session data
    const sessionData: SessionData = JSON.parse(sessionCookie.value);
    
    // Check if session is expired
    const now = Math.floor(Date.now() / 1000);
    if (sessionData.expiresAt && sessionData.expiresAt < now) {
      // Try to refresh token if available
      const refreshToken = cookieStore.get('admin_refresh_token');
      if (refreshToken?.value) {
        try {
          const newSession = await refreshSession(refreshToken.value);
          if (newSession) {
            await updateSessionCookies(cookieStore, newSession);
            return NextResponse.json({
              authenticated: true,
              user: {
                id: newSession.userId,
                email: newSession.email,
                name: newSession.name,
                picture: newSession.picture,
                role: newSession.role,
                permissions: newSession.permissions,
                emailVerified: newSession.emailVerified,
                sessionId: newSession.sessionId,
              }
            });
          }
        } catch (error) {
          console.error('Token refresh failed:', error);
        }
      }
      
      // Session expired and couldn't refresh
      return NextResponse.json(
        { authenticated: false, error: 'Session expired' },
        { status: 401 }
      );
    }
    
    // Verify admin permissions
    if (!hasAdminPermissions(sessionData)) {
      return NextResponse.json(
        { authenticated: false, error: 'Insufficient permissions' },
        { status: 403 }
      );
    }
    
    // Optionally validate token with backend
    if (accessToken?.value) {
      const isValid = await validateTokenWithBackend(accessToken.value);
      if (!isValid) {
        return NextResponse.json(
          { authenticated: false, error: 'Invalid token' },
          { status: 401 }
        );
      }
    }
    
    // Return session information
    return NextResponse.json({
      authenticated: true,
      user: {
        id: sessionData.userId,
        email: sessionData.email,
        name: sessionData.name,
        picture: sessionData.picture,
        role: sessionData.role,
        permissions: sessionData.permissions,
        emailVerified: sessionData.emailVerified,
        sessionId: sessionData.sessionId,
      }
    });
    
  } catch (error) {
    console.error('🚨 Session validation error:', error);
    return NextResponse.json(
      { authenticated: false, error: 'Session validation failed' },
      { status: 500 }
    );
  }
}

/**
 * Check if user has required admin permissions
 */
function hasAdminPermissions(sessionData: SessionData): boolean {
  // First check legacy admin roles for backward compatibility
  const adminRoles = [
    'super_admin',
    'admin-full-004',
    'moderator-standard-003',
    'admin',
    'moderator'
  ];
  
  const hasLegacyAdmin = adminRoles.includes(sessionData.role) || 
                        sessionData.permissions.some(permission => 
                          permission.startsWith('admin:') || permission === 'admin'
                        );
  
  if (hasLegacyAdmin) {
    return true;
  }
  
  // TODO: Check granular admin modules via backend API
  // For now, allow in development mode if email exists (granular checking happens at component level)
  if (process.env.NODE_ENV === 'development' && sessionData.email) {
    return true;
  }
  
  return false;
}

/**
 * Validate token with backend
 */
async function validateTokenWithBackend(accessToken: string): Promise<boolean> {
  try {
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    const response = await fetch(`${backendUrl}/oauth/userinfo`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Accept': 'application/json',
      },
    });
    
    return response.ok;
  } catch (error) {
    console.error('Backend token validation failed:', error);
    return false;
  }
}

/**
 * Refresh session using refresh token
 */
async function refreshSession(refreshToken: string): Promise<SessionData | null> {
  try {
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    const response = await fetch(`${backendUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: process.env.OIDC_CLIENT_ID || 'epsx-admin',
      }).toString(),
    });
    
    if (!response.ok) {
      return null;
    }
    
    const tokenData = await response.json();
    
    // Get updated user info
    const userInfoResponse = await fetch(`${backendUrl}/oauth/userinfo`, {
      headers: {
        'Authorization': `Bearer ${tokenData.access_token}`,
      },
    });
    
    if (!userInfoResponse.ok) {
      return null;
    }
    
    const userInfo = await userInfoResponse.json();
    
    return {
      userId: userInfo.sub,
      email: userInfo.email,
      name: userInfo.name,
      picture: userInfo.picture,
      role: userInfo.role,
      permissions: userInfo.permissions,
      emailVerified: userInfo.email_verified,
      sessionId: `admin_${Date.now()}_${Math.random().toString(36).substring(2)}`,
      issuedAt: Math.floor(Date.now() / 1000),
      expiresAt: Math.floor(Date.now() / 1000) + (tokenData.expires_in || 28800), // 8 hours
    };
    
  } catch (error) {
    console.error('Session refresh failed:', error);
    return null;
  }
}

/**
 * Update session cookies with refreshed data
 */
async function updateSessionCookies(cookieStore: any, sessionData: SessionData): Promise<void> {
  const isProduction = process.env.NODE_ENV === 'production';
  const maxAge = sessionData.expiresAt - sessionData.issuedAt;
  
  cookieStore.set('admin_session', JSON.stringify(sessionData), {
    httpOnly: true,
    secure: isProduction,
    sameSite: 'lax',
    maxAge: maxAge,
    path: '/',
    domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
  });
}