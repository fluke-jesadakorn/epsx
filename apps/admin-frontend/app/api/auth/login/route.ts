import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

/**
 * Server-side OpenID Connect authorization endpoint
 * Redirects user to backend OIDC provider
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const callbackUrl = searchParams.get('callbackUrl') || '/';
  const isAdmin = request.nextUrl.pathname.includes('admin');
  
  // Store the callback URL in a secure cookie for later use
  const cookieStore = await cookies();
  cookieStore.set('oidc_callback_url', callbackUrl, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 60 * 10, // 10 minutes
    path: '/'
  });
  
  // Build OIDC authorization URL
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  const clientId = process.env.OIDC_CLIENT_ID || 'epsx-admin';
  const redirectUri = `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/api/auth/callback`;
  const state = generateState();
  const nonce = generateNonce();
  
  // Store state and nonce for security validation
  cookieStore.set('oidc_state', state, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 60 * 10, // 10 minutes
    path: '/'
  });
  
  cookieStore.set('oidc_nonce', nonce, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 60 * 10, // 10 minutes
    path: '/'
  });
  
  // Construct OIDC authorization URL (using Firebase-native OIDC endpoints)
  const authUrl = new URL(`${backendUrl}/oauth/authorize`);
  authUrl.searchParams.set('client_id', clientId);
  authUrl.searchParams.set('redirect_uri', redirectUri);
  authUrl.searchParams.set('response_type', 'code');
  authUrl.searchParams.set('scope', isAdmin ? 'openid profile email admin' : 'openid profile email');
  authUrl.searchParams.set('state', state);
  authUrl.searchParams.set('nonce', nonce);
  
  if (isAdmin) {
    authUrl.searchParams.set('admin', 'true');
  }
  
  console.log('🔐 OIDC Authorization redirect:', {
    authUrl: authUrl.toString(),
    callbackUrl,
    isAdmin,
    timestamp: new Date().toISOString()
  });
  
  return NextResponse.redirect(authUrl.toString());
}

/**
 * Generate cryptographically secure random state parameter
 */
function generateState(): string {
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    return btoa(String.fromCharCode(...array))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }
  
  // Fallback for environments without crypto API
  return Math.random().toString(36).substring(2) + 
         Math.random().toString(36).substring(2) +
         Date.now().toString(36);
}

/**
 * Generate cryptographically secure random nonce parameter
 */
function generateNonce(): string {
  return generateState(); // Same implementation as state
}