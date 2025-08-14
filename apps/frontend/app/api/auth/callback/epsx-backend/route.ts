/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens, getUserInfo } from '@/lib/auth/client';
import { createUserSession } from '@/lib/auth/session';
import { setSessionCookie } from '@/lib/auth/session';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    console.log('🔄 Frontend: EPSX Backend OAuth callback received:', {
      code: code ? 'present' : 'missing',
      state: state ? 'present' : 'missing',
      error,
      errorDescription,
    });

    // Handle OAuth errors
    if (error) {
      console.error('❌ Frontend: OAuth callback error:', error, errorDescription);
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', error);
      return NextResponse.redirect(loginUrl);
    }

    // Validate required parameters
    if (!code || !state) {
      console.error('❌ Frontend: Missing required OAuth parameters');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_parameters');
      return NextResponse.redirect(loginUrl);
    }

    // Retrieve PKCE parameters from cookies
    const cookieStore = await cookies();
    const storedCodeVerifier = cookieStore.get('oauth_code_verifier')?.value;
    const storedState = cookieStore.get('oauth_state')?.value;

    if (!storedCodeVerifier || !storedState) {
      console.error('❌ Frontend: Missing PKCE parameters in cookies');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_pkce');
      return NextResponse.redirect(loginUrl);
    }

    // Validate state parameter
    if (state !== storedState) {
      console.error('❌ Frontend: State parameter mismatch');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'state_mismatch');
      return NextResponse.redirect(loginUrl);
    }

    console.log('🔄 Frontend: Exchanging authorization code for tokens with EPSX backend');

    // Exchange authorization code for tokens
    const { accessToken, idToken, refreshToken } = await exchangeCodeForTokens(
      code,
      storedCodeVerifier,
      state
    );

    console.log('✅ Frontend: Successfully received tokens from EPSX backend');

    // Get user information from userinfo endpoint
    console.log('🔄 Frontend: Fetching user information from EPSX backend');
    const userinfo = await getUserInfo(accessToken);

    console.log('✅ Frontend: Successfully received user info from EPSX backend:', {
      email: userinfo.email,
      role: userinfo.role,
      permissions: userinfo.permissions,
      package_tier: userinfo.package_tier,
    });

    // Create user session
    const sessionData = createUserSession(userinfo, accessToken, refreshToken);

    console.log('✅ Frontend: User session created successfully');

    // Get callback URL from cookies or default to dashboard
    const callbackUrl = cookieStore.get('oauth_callback_url')?.value || '/dashboard';
    
    // Clean up OAuth cookies and create redirect response
    const response = NextResponse.redirect(new URL(callbackUrl, request.url));
    
    // Use manual session cookie setting instead of iron-session
    console.log('🔧 Frontend: Setting session cookie manually for redirect...');
    await setSessionCookie(response, sessionData);
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');

    console.log('✅ Frontend: Redirecting to:', callbackUrl);
    return response;

  } catch (error) {
    console.error('❌ Frontend: EPSX Backend OAuth callback processing error:', error);
    
    // Log detailed error for debugging
    if (error instanceof Error) {
      console.error('❌ Frontend: Error details:', {
        message: error.message,
        stack: error.stack,
      });
    }

    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'callback_error');
    
    return NextResponse.redirect(loginUrl);
  }
}