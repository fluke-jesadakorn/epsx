/**
 * OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens, getUserInfo } from '@/lib/auth/client';
import { createUserSession, SessionData } from '@/lib/auth/session';
import { setSessionCookie } from '@/lib/auth/session';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  // FIRST: Log that route is called
  console.log('🚨 CALLBACK ROUTE CALLED - START');
  console.log('🚨 Request URL:', request.url);
  console.log('🚨 Time:', new Date().toISOString());
  
  try {
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    console.log('🔄 EPSX Backend OAuth callback received:', {
      code: code ? 'present' : 'missing',
      state: state ? 'present' : 'missing',
      error,
      errorDescription,
    });

    // Handle OAuth errors
    if (error) {
      console.error('❌ OAuth callback error:', error, errorDescription);
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', error);
      return NextResponse.redirect(loginUrl);
    }

    // Validate required parameters
    if (!code || !state) {
      console.error('❌ Missing required OAuth parameters');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_parameters');
      return NextResponse.redirect(loginUrl);
    }

    // Retrieve PKCE parameters from cookies
    const cookieStore = await cookies();
    const storedCodeVerifier = cookieStore.get('oauth_code_verifier')?.value;
    const storedState = cookieStore.get('oauth_state')?.value;

    if (!storedCodeVerifier || !storedState) {
      console.error('❌ Missing PKCE parameters in cookies');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_pkce');
      return NextResponse.redirect(loginUrl);
    }

    // Validate state parameter
    if (state !== storedState) {
      console.error('❌ State parameter mismatch');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'state_mismatch');
      return NextResponse.redirect(loginUrl);
    }

    console.log('🔄 Exchanging authorization code for tokens with EPSX backend');

    // Exchange authorization code for tokens
    const { accessToken, idToken, refreshToken } = await exchangeCodeForTokens(
      code,
      storedCodeVerifier,
      state
    );

    console.log('✅ Successfully received tokens from EPSX backend');

    // Get user information from userinfo endpoint
    console.log('🔄 Fetching user information from EPSX backend');
    const userinfo = await getUserInfo(accessToken);

    console.log('✅ Successfully received user info from EPSX backend:', {
      email: userinfo.email,
      role: userinfo.role,
      admin_modules: userinfo.admin_modules,
    });

    // Create user session data
    console.log('🔄 Creating user session with userinfo:', { email: userinfo.email, role: userinfo.role });
    const sessionData = createUserSession(userinfo, accessToken, refreshToken);
    console.log('✅ User session created successfully');

    // Redirect to dashboard
    const dashboardUrl = new URL('/', request.url);
    const response = NextResponse.redirect(dashboardUrl);
    
    // Use manual session cookie setting
    console.log('🔧 Setting session cookie manually for redirect...');
    try {
      setSessionCookie(response, sessionData);
      console.log('✅ Manual session cookie set successfully');
    } catch (sessionError) {
      console.error('❌ Manual session cookie error:', sessionError);
      throw sessionError;
    }
    
    // Clean up OAuth cookies in the response
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    
    console.log('✅ Callback completed successfully, redirecting with clean cookies');
    console.log('🚨 CALLBACK ROUTE CALLED - END');
    return response;

  } catch (error) {
    console.error('❌ EPSX Backend OAuth callback processing error:', error);
    
    // Log detailed error for debugging
    if (error instanceof Error) {
      console.error('❌ Error details:', {
        message: error.message,
        stack: error.stack,
      });
    }

    // Enhanced error logging
    console.error('❌ Full error object:', JSON.stringify(error, null, 2));

    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    return NextResponse.redirect(loginUrl);
  }
}