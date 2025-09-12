/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens } from '@/lib/server/auth';
import { cookies } from 'next/headers';
import { logger, devLog, safeError } from '@/lib/logger';

export async function GET(request: NextRequest) {
  devLog('FRONTEND CALLBACK ROUTE CALLED - START');
  devLog('Request URL: ' + request.url);
  devLog('Time: ' + new Date().toISOString());
  
  try {
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    devLog('Frontend: EPSX Backend OAuth callback received', {
      code: code ? 'present' : 'missing',
      state: state ? 'present' : 'missing',
      error,
      errorDescription,
    });

    // Handle OAuth errors
    if (error) {
      logger.error('Frontend: OAuth callback error', { error, errorDescription });
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', error);
      if (errorDescription) {
        loginUrl.searchParams.set('error_description', errorDescription);
      }
      return NextResponse.redirect(loginUrl);
    }

    // Validate required parameters
    if (!code || !state) {
      logger.error('Frontend: Missing required OAuth parameters');
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
    devLog('Frontend Cookie debug info', {
      oauth_code_verifier: storedCodeVerifier ? `present (${storedCodeVerifier.slice(0,10)}...)` : 'missing',
      oauth_state: storedState ? `present (${storedState.slice(0,10)}...)` : 'missing',
      oauth_callback_url: storedCallbackUrl || 'missing'
    });

    // Validate PKCE parameters are present
    if (!storedCodeVerifier || !storedState) {
      logger.error('Frontend: Missing PKCE parameters in cookies');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'missing_pkce_parameters');
      return NextResponse.redirect(loginUrl);
    }

    // Validate state parameter
    devLog('Frontend State validation', {
      received_state: state,
      stored_state: storedState,
      states_match: state === storedState
    });
    
    if (state !== storedState) {
      logger.error('Frontend: State mismatch detected');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'state_mismatch');
      return NextResponse.redirect(loginUrl);
    }
    
    devLog('Frontend: State validation passed');

    // Exchange authorization code for OIDC tokens using backend OIDC service
    devLog('Frontend: Exchanging authorization code for OIDC tokens...');
    const tokens = await exchangeCodeForTokens(code, storedCodeVerifier, state);
    
    // OIDC Migration: Receive all three standard OIDC tokens
    const accessToken = tokens.accessToken;   // Bearer token for API access
    const refreshToken = tokens.refreshToken; // Token for refreshing access
    const idToken = tokens.idToken;           // User identity claims
    
    devLog('Frontend: Successfully received OIDC tokens', {
      accessToken: accessToken ? 'present' : 'missing',
      refreshToken: refreshToken ? 'present' : 'missing', 
      idToken: idToken ? 'present' : 'missing'
    });

    // Validate that we received all required OIDC tokens
    if (!accessToken || !refreshToken || !idToken) {
      logger.error('Frontend: Missing required OIDC tokens');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'incomplete_oidc_tokens');
      return NextResponse.redirect(loginUrl);
    }

    // Get callback URL from cookies or default to home page
    const callbackUrl = storedCallbackUrl || '/';
    
    devLog('Frontend: Redirecting to callback URL: ' + callbackUrl);
    const redirectUrl = new URL(callbackUrl, request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // Clean up OAuth cookies FIRST
    devLog('Frontend: Cleaning up OAuth cookies...');
    if (storedCodeVerifier) {
      response.cookies.delete('oauth_code_verifier');
      devLog('Cleaned oauth_code_verifier cookie');
    }
    if (storedState) {
      response.cookies.delete('oauth_state');
      devLog('Cleaned oauth_state cookie');  
    }
    if (storedCallbackUrl) {
      response.cookies.delete('oauth_callback_url');
      devLog('Cleaned oauth_callback_url cookie');
    }
    
    // OIDC Migration: Set standard OIDC cookies instead of custom JWT
    devLog('Frontend: Setting OIDC-compliant cookies...');
    
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
    devLog('OIDC access_token cookie set (1 hour expiry)');
    
    // ID Token - Contains user identity claims (same as access token)
    response.cookies.set('id_token', idToken, {
      ...cookieOptions,
      maxAge: 60 * 60, // 1 hour
    });
    devLog('OIDC id_token cookie set (1 hour expiry)');
    
    // Refresh Token - Long-lived (30 days)
    response.cookies.set('refresh_token', refreshToken, {
      ...cookieOptions,
      maxAge: 30 * 24 * 60 * 60, // 30 days
    });
    devLog('OIDC refresh_token cookie set (30 days expiry)');

    // Clean up legacy JWT cookie if it exists
    response.cookies.delete('epsx_frontend_jwt');
    devLog('Legacy JWT cookie cleaned up');

    devLog('Frontend: Callback completed successfully with OIDC cookies');
    devLog('FRONTEND CALLBACK ROUTE CALLED - END');
    return response;

  } catch (error) {
    logger.error('Frontend: EPSX Backend OAuth callback processing error', error);
    
    // Log detailed error for debugging
    if (error instanceof Error) {
      logger.error('Frontend: Error details', {
        message: error.message,
        stack: error.stack,
      });
    }

    // Redirect to login page with error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    devLog('FRONTEND CALLBACK ROUTE CALLED - END (ERROR)');
    return NextResponse.redirect(loginUrl);
  }
}