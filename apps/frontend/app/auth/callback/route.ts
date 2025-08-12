import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

/**
 * OIDC Authentication Callback Route Handler
 * Handles OAuth2/OIDC authorization code exchange for frontend users
 * 
 * Features:
 * - Server-side code exchange for enhanced security
 * - HTTP-only cookie storage for tokens
 * - Clean error handling and redirects
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const code = searchParams.get('code');
  const state = searchParams.get('state');
  const error = searchParams.get('error');
  const error_description = searchParams.get('error_description');

  // Handle OAuth error responses
  if (error) {
    console.error('OIDC Auth Error:', { error, error_description });
    return NextResponse.redirect(new URL(`/login?error=${encodeURIComponent(error_description || error)}`, request.url));
  }

  // Validate required parameters
  if (!code) {
    console.error('OIDC: Missing authorization code');
    return NextResponse.redirect(new URL('/login?error=missing_authorization_code', request.url));
  }

  try {
    // Exchange authorization code for tokens via backend
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const tokenEndpoint = `${backendUrl}/oauth/token`;
    
    const tokenResponse = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'Authorization': `Basic ${Buffer.from(
          `epsx-frontend:${process.env.FRONTEND_CLIENT_SECRET || 'frontend-client-secret'}`
        ).toString('base64')}`,
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code: code,
        redirect_uri: `${process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000'}/auth/callback`,
        client_id: 'epsx-frontend',
      }),
    });

    if (!tokenResponse.ok) {
      const errorText = await tokenResponse.text();
      console.error('Token exchange failed:', errorText);
      return NextResponse.redirect(new URL('/login?error=token_exchange_failed', request.url));
    }

    const tokenData = await tokenResponse.json();
    const { access_token, refresh_token, id_token } = tokenData;

    // Create response and set HTTP-only cookies for secure token storage
    const response = NextResponse.redirect(new URL(state && state !== '/' ? state : '/dashboard', request.url));
    const cookieStore = await cookies();
    
    // Set main auth token
    cookieStore.set('auth-token', access_token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 60 * 2, // 2 hours
      path: '/',
    });

    // Set refresh token if provided
    if (refresh_token) {
      cookieStore.set('refresh-token', refresh_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 7, // 7 days
        path: '/',
      });
    }

    // Set ID token if provided (for user info)
    if (id_token) {
      cookieStore.set('id-token', id_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 2, // 2 hours
        path: '/',
      });
    }

    // Log successful authentication
    console.log('🔐 User authentication successful', {
      timestamp: new Date().toISOString(),
      client_id: 'epsx-frontend',
      // Note: Don't log actual tokens for security
      has_access_token: !!access_token,
      has_refresh_token: !!refresh_token,
      has_id_token: !!id_token,
    });

    return response;

  } catch (error) {
    console.error('OIDC callback error:', error);
    return NextResponse.redirect(new URL('/login?error=authentication_failed', request.url));
  }
}