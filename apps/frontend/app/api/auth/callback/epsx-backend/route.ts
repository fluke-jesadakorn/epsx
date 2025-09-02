/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens } from '@/lib/server/auth';
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

    // Exchange authorization code for OIDC tokens using backend OIDC service
    console.log('🔄 Frontend: Exchanging authorization code for OIDC tokens...');
    const tokens = await exchangeCodeForTokens(code, storedCodeVerifier, state);
    
    // OIDC Migration: Receive all three standard OIDC tokens
    const accessToken = tokens.accessToken;   // Bearer token for API access
    const refreshToken = tokens.refreshToken; // Token for refreshing access
    const idToken = tokens.idToken;           // User identity claims
    
    console.log('✅ Frontend: Successfully received OIDC tokens:', {
      accessToken: accessToken ? 'present' : 'missing',
      refreshToken: refreshToken ? 'present' : 'missing', 
      idToken: idToken ? 'present' : 'missing'
    });

    // Validate that we received all required OIDC tokens
    if (!accessToken || !refreshToken || !idToken) {
      console.error('❌ Frontend: Missing required OIDC tokens');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'incomplete_oidc_tokens');
      return NextResponse.redirect(loginUrl);
    }

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
    
    // OIDC Migration: Set standard OIDC cookies instead of custom JWT
    console.log('🔧 Frontend: Setting OIDC-compliant cookies...');
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/'
    };
    
    // Access Token - Short-lived (1 hour)
    response.cookies.set('access_token', accessToken, {
      ...cookieOptions,
      maxAge: 60 * 60, // 1 hour
    });
    console.log('✅ OIDC access_token cookie set (1 hour expiry)');
    
    // ID Token - Contains user identity claims (same as access token)
    response.cookies.set('id_token', idToken, {
      ...cookieOptions,
      maxAge: 60 * 60, // 1 hour
    });
    console.log('✅ OIDC id_token cookie set (1 hour expiry)');
    
    // Refresh Token - Long-lived (30 days)
    response.cookies.set('refresh_token', refreshToken, {
      ...cookieOptions,
      maxAge: 30 * 24 * 60 * 60, // 30 days
    });
    console.log('✅ OIDC refresh_token cookie set (30 days expiry)');

    // Clean up legacy JWT cookie if it exists
    response.cookies.delete('epsx_frontend_jwt');
    console.log('✅ Legacy JWT cookie cleaned up');

    console.log('✅ Frontend: Callback completed successfully with OIDC cookies');
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