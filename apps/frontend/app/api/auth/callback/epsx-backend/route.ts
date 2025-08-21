/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens, getUserInfo } from '@/lib/server/auth';
import { signJWT, createJWTClaims } from '@/lib/auth-utils';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  console.log('🚨 FRONTEND CALLBACK ROUTE CALLED - START');
  console.log('🚨 Request URL:', request.url);
  console.log('🚨 Time:', new Date().toISOString());
  
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
      if (errorDescription) {
        loginUrl.searchParams.set('error_description', errorDescription);
      }
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
    const storedCallbackUrl = cookieStore.get('oauth_callback_url')?.value;

    // Debug cookie values
    console.log('🔍 Frontend Cookie debug info:');
    console.log('  - oauth_code_verifier:', storedCodeVerifier ? `present (${storedCodeVerifier.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_state:', storedState ? `present (${storedState.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_callback_url:', storedCallbackUrl || 'missing');

    // Validate PKCE parameters are present
    if (!storedCodeVerifier || !storedState) {
      console.error('❌ Frontend: Missing PKCE parameters in cookies');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_pkce_parameters');
      return NextResponse.redirect(loginUrl);
    }

    // Validate state parameter
    console.log('🔍 Frontend State validation:');
    console.log('  - Received state:', state);
    console.log('  - Stored state:', storedState);
    console.log('  - States match:', state === storedState);
    
    if (state !== storedState) {
      console.error('❌ Frontend: State mismatch detected');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'state_mismatch');
      return NextResponse.redirect(loginUrl);
    }
    
    console.log('✅ Frontend: State validation passed');

    // Exchange authorization code for access token using PKCE
    console.log('🔄 Frontend: Exchanging authorization code for tokens...');
    const tokens = await exchangeCodeForTokens(code, storedCodeVerifier, state);
    const accessToken = tokens.accessToken;
    console.log('✅ Frontend: Successfully exchanged code for access token');

    // Get user information from userinfo endpoint
    console.log('🔄 Frontend: Fetching user information from EPSX backend');
    const userinfo = await getUserInfo(accessToken);

    console.log('✅ Frontend: Successfully received user info from EPSX backend:', {
      email: userinfo.email,
      role: userinfo.role,
      permissions: userinfo.permissions,
      package_tier: userinfo.subscription_tier,
    });

    // Create JWT token with user claims
    const jwtClaims = createJWTClaims({
      id: userinfo.sub || userinfo.id,
      email: userinfo.email,
      name: userinfo.name || userinfo.display_name,
      admin_modules: userinfo.admin_modules || [],
      permissions: userinfo.permissions || ['user:read'],
      package_tier: userinfo.subscription_tier || 'FREE', // Use subscription_tier from backend
      role: userinfo.role || 'user',
      firebase_uid: userinfo.firebase_uid || userinfo.sub,
    });

    const jwtToken = await signJWT(jwtClaims);

    console.log('✅ Frontend: JWT token created successfully');

    // Get callback URL from cookies or default to dashboard
    const callbackUrl = storedCallbackUrl || '/dashboard';
    
    console.log('🔄 Frontend: Redirecting to callback URL:', callbackUrl);
    const redirectUrl = new URL(callbackUrl, request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // Clean up OAuth cookies FIRST
    console.log('🔧 Frontend: Cleaning up OAuth cookies...');
    if (storedCodeVerifier) {
      response.cookies.delete('oauth_code_verifier');
      console.log('✅ Cleaned oauth_code_verifier cookie');
    }
    if (storedState) {
      response.cookies.delete('oauth_state');
      console.log('✅ Cleaned oauth_state cookie');  
    }
    if (storedCallbackUrl) {
      response.cookies.delete('oauth_callback_url');
      console.log('✅ Cleaned oauth_callback_url cookie');
    }
    
    // Set JWT cookie AFTER cleanup
    console.log('🔧 Frontend: Setting JWT cookie for redirect...');
    response.cookies.set('epsx_frontend_jwt', jwtToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 24 * 60 * 60, // 24 hours
      path: '/'
    });
    console.log('✅ Frontend JWT cookie set successfully');

    console.log('✅ Frontend: Callback completed successfully, redirecting with clean cookies');
    console.log('🚨 FRONTEND CALLBACK ROUTE CALLED - END');
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

    // Enhanced error logging
    console.error('❌ Frontend: Full error object:', JSON.stringify(error, null, 2));

    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    console.log('🚨 FRONTEND CALLBACK ROUTE CALLED - END (ERROR)');
    return NextResponse.redirect(loginUrl);
  }
}