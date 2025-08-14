/**
 * OAuth Login Route
 * Initiates OAuth flow with backend OIDC provider
 */
import { NextRequest, NextResponse } from 'next/server';
import { getAuthorizationUrl } from '@/lib/auth/client';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    console.log('🚀 Starting OAuth login flow');

    // Generate OAuth authorization URL with PKCE
    const { url, codeVerifier, state } = await getAuthorizationUrl();

    // Store PKCE parameters in secure cookies for callback
    const cookieStore = await cookies();
    cookieStore.set('oauth_code_verifier', codeVerifier, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 10, // 10 minutes
    });
    
    cookieStore.set('oauth_state', state, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 10, // 10 minutes
    });

    console.log('🚀 Redirecting to OAuth authorization URL:', url);

    // Redirect to OAuth provider
    return NextResponse.redirect(url);
  } catch (error) {
    console.error('❌ OAuth login error:', error);
    
    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'oauth_error');
    
    return NextResponse.redirect(loginUrl);
  }
}

export async function POST(request: NextRequest) {
  // Also support POST for login forms
  return GET(request);
}