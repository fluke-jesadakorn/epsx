import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

interface OIDCTokenResponse {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
  scope: string;
}

interface OIDCUserInfo {
  sub: string;
  email: string;
  email_verified: boolean;
  name?: string;
  picture?: string;
  role: string;
  permissions: string[];
  iat: number;
  exp: number;
}

/**
 * Server-side OpenID Connect callback handler
 * Exchanges authorization code for tokens and establishes session
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const code = searchParams.get('code');
  const state = searchParams.get('state');
  const error = searchParams.get('error');
  const cookieStore = await cookies();
  
  // Handle OIDC errors
  if (error) {
    console.error('🚨 OIDC Error:', error);
    return NextResponse.redirect(
      `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login?error=${encodeURIComponent(error)}`
    );
  }
  
  // Validate required parameters
  if (!code || !state) {
    console.error('🚨 OIDC Callback: Missing code or state parameter');
    return NextResponse.redirect(
      `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login?error=invalid_request`
    );
  }
  
  // Validate state parameter (CSRF protection)
  const storedState = cookieStore.get('oidc_state')?.value;
  if (!storedState || storedState !== state) {
    console.error('🚨 OIDC Callback: Invalid state parameter');
    return NextResponse.redirect(
      `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login?error=invalid_state`
    );
  }
  
  try {
    // Exchange authorization code for tokens
    const tokenResponse = await exchangeCodeForTokens(code);
    
    // Validate and decode ID token
    const userInfo = await validateIdToken(tokenResponse.id_token);
    
    // Verify admin permissions
    if (!hasAdminPermissions(userInfo)) {
      console.warn('🚨 OIDC Callback: User lacks admin permissions:', userInfo.email);
      return NextResponse.redirect(
        `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login?error=insufficient_permissions`
      );
    }
    
    // Create secure session cookies
    await createSessionCookies(cookieStore, tokenResponse, userInfo);
    
    // Get callback URL and redirect
    const callbackUrl = cookieStore.get('oidc_callback_url')?.value || '/';
    
    // Clean up temporary cookies
    cookieStore.delete('oidc_state');
    cookieStore.delete('oidc_nonce');
    cookieStore.delete('oidc_callback_url');
    
    console.log('✅ OIDC Authentication successful:', {
      userId: userInfo.sub,
      email: userInfo.email,
      role: userInfo.role,
      callbackUrl,
      timestamp: new Date().toISOString()
    });
    
    return NextResponse.redirect(callbackUrl);
    
  } catch (error) {
    console.error('🚨 OIDC Callback Error:', error);
    return NextResponse.redirect(
      `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login?error=authentication_failed`
    );
  }
}

/**
 * Exchange authorization code for access and ID tokens
 */
async function exchangeCodeForTokens(code: string): Promise<OIDCTokenResponse> {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  const clientId = process.env.OIDC_CLIENT_ID || 'epsx-admin';
  const clientSecret = process.env.OIDC_CLIENT_SECRET;
  const redirectUri = `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/api/auth/callback`;
  
  const tokenUrl = `${backendUrl}/oauth/token`;
  
  const body = new URLSearchParams({
    grant_type: 'authorization_code',
    code: code,
    redirect_uri: redirectUri,
    client_id: clientId,
  });
  
  if (clientSecret) {
    body.append('client_secret', clientSecret);
  }
  
  const response = await fetch(tokenUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      'Accept': 'application/json',
    },
    body: body.toString(),
  });
  
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Token exchange failed: ${response.status} ${errorText}`);
  }
  
  return await response.json() as OIDCTokenResponse;
}

/**
 * Validate and decode ID token
 */
async function validateIdToken(idToken: string): Promise<OIDCUserInfo> {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  const response = await fetch(`${backendUrl}/api/auth/oidc/userinfo`, {
    headers: {
      'Authorization': `Bearer ${idToken}`,
      'Accept': 'application/json',
    },
  });
  
  if (!response.ok) {
    throw new Error(`UserInfo request failed: ${response.status}`);
  }
  
  return await response.json() as OIDCUserInfo;
}

/**
 * Check if user has required admin permissions
 */
function hasAdminPermissions(userInfo: OIDCUserInfo): boolean {
  const adminRoles = [
    'super_admin',
    'admin-full-004',
    'moderator-standard-003',
    'admin',
    'moderator'
  ];
  
  return adminRoles.includes(userInfo.role) || 
         userInfo.permissions.some(permission => 
           permission.startsWith('admin:') || permission === 'admin'
         );
}

/**
 * Create secure session cookies
 */
async function createSessionCookies(
  cookieStore: any,
  tokenResponse: OIDCTokenResponse,
  userInfo: OIDCUserInfo
): Promise<void> {
  const isProduction = process.env.NODE_ENV === 'production';
  const maxAge = tokenResponse.expires_in || (60 * 60 * 8); // 8 hours default
  
  // Create session data
  const sessionData = {
    userId: userInfo.sub,
    email: userInfo.email,
    name: userInfo.name,
    picture: userInfo.picture,
    role: userInfo.role,
    permissions: userInfo.permissions,
    emailVerified: userInfo.email_verified,
    sessionId: generateSessionId(),
    issuedAt: Math.floor(Date.now() / 1000),
    expiresAt: Math.floor(Date.now() / 1000) + maxAge,
  };
  
  // Set session cookie
  cookieStore.set('admin_session', JSON.stringify(sessionData), {
    httpOnly: true,
    secure: isProduction,
    sameSite: 'lax',
    maxAge: maxAge,
    path: '/',
    domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
  });
  
  // Set access token cookie (for API requests)
  cookieStore.set('admin_access_token', tokenResponse.access_token, {
    httpOnly: true,
    secure: isProduction,
    sameSite: 'lax',
    maxAge: maxAge,
    path: '/',
    domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
  });
  
  // Set refresh token if available
  if (tokenResponse.refresh_token) {
    cookieStore.set('admin_refresh_token', tokenResponse.refresh_token, {
      httpOnly: true,
      secure: isProduction,
      sameSite: 'lax',
      maxAge: maxAge * 24, // Refresh tokens typically last longer
      path: '/',
      domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
    });
  }
}

/**
 * Generate unique session ID
 */
function generateSessionId(): string {
  return `admin_${Date.now()}_${Math.random().toString(36).substring(2)}`;
}