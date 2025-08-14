/**
 * Frontend Sign In API Route
 * Initiates OAuth 2.0 authorization flow for trading platform backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { getAuthorizationUrl } from '@/lib/auth';

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Initiating OAuth sign-in flow');

    // Get callback URL from query parameters
    const { searchParams } = new URL(request.url);
    const callbackUrl = searchParams.get('callbackUrl') || '/';

    // Generate authorization URL with PKCE parameters
    const { url, codeVerifier, state } = await getAuthorizationUrl();

    console.log('✅ Frontend: Generated authorization URL');

    // Create response that redirects to authorization server
    const response = NextResponse.redirect(url);

    // Store PKCE parameters in secure cookies
    response.cookies.set('oauth_code_verifier', codeVerifier, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 600, // 10 minutes
      path: '/',
    });

    response.cookies.set('oauth_state', state, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 600, // 10 minutes
      path: '/',
    });

    // Store callback URL for post-authentication redirect
    response.cookies.set('oauth_callback_url', callbackUrl, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 600, // 10 minutes
      path: '/',
    });

    console.log('🔄 Frontend: Redirecting to EPSX backend authorization server with callback URL:', callbackUrl);
    return response;

  } catch (error) {
    console.error('❌ Frontend: Sign-in initiation failed:', error);
    
    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'signin_failed');
    
    return NextResponse.redirect(loginUrl);
  }
}

// Allow POST method as well for form submissions
export async function POST(request: NextRequest) {
  return GET(request);
}