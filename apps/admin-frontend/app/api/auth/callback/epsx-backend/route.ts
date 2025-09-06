/**
 * OIDC Callback Route for EPSX Backend
 * Handles OIDC authorization callback and sets OIDC tokens as HttpOnly cookies
 * OIDC Migration: Uses standard OIDC tokens instead of custom JWT
 */
import { NextRequest, NextResponse } from 'next/server';
import { getUserInfo, exchangeCodeForTokens } from '@/lib/server/auth';
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
    let storedCodeVerifier = cookieStore.get('oauth_code_verifier')?.value;
    let storedState = cookieStore.get('oauth_state')?.value;
    const storedCallbackUrl = cookieStore.get('oauth_callback_url')?.value;

    // If primary cookies are missing, try backup cookies
    if (!storedCodeVerifier) {
      storedCodeVerifier = cookieStore.get('pkce_verifier_backup')?.value;
      console.log('🔄 Using backup code verifier');
    }
    
    if (!storedState) {
      storedState = cookieStore.get('pkce_state_backup')?.value;
      console.log('🔄 Using backup state');
    }

    // Debug cookie values
    console.log('🔍 Admin Cookie debug info:');
    console.log('  - oauth_code_verifier:', storedCodeVerifier ? `present (${storedCodeVerifier.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_state:', storedState ? `present (${storedState.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_callback_url:', storedCallbackUrl || 'missing');

    // Debug all cookies to see what's available
    const allCookies = cookieStore.getAll();
    console.log('🔍 All available cookies:', allCookies.map(c => c.name).join(', '));

    // Validate PKCE parameters are present
    if (!storedCodeVerifier || !storedState) {
      console.error('❌ Admin: Missing PKCE parameters in cookies');
      console.error('❌ Available cookies:', allCookies.map(c => `${c.name}=${c.value ? 'present' : 'empty'}`));
      
      // Try to restart the OAuth flow instead of showing error
      console.log('🔄 Attempting to restart OAuth flow...');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('redirectTo', request.nextUrl.pathname);
      return NextResponse.redirect(loginUrl);
    }

    // Validate state parameter
    console.log('🔍 Admin State validation:');
    console.log('  - Received state:', state);
    console.log('  - Stored state:', storedState);
    console.log('  - States match:', state === storedState);
    
    if (state !== storedState) {
      console.error('❌ Admin: State mismatch detected');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('error', 'state_mismatch');
      return NextResponse.redirect(loginUrl);
    }
    
    console.log('✅ Admin: State validation passed');

    // OIDC Migration: Exchange authorization code for OIDC tokens
    console.log('🔄 Admin: Exchanging authorization code for OIDC tokens...');
    const tokens = await exchangeCodeForTokens(code, storedCodeVerifier, state);
    console.log('✅ Admin: Successfully exchanged authorization code for OIDC tokens:', {
      hasAccessToken: !!tokens.accessToken,
      hasIdToken: !!tokens.idToken,
      hasRefreshToken: !!tokens.refreshToken,
      expiresIn: tokens.expiresIn
    });

    // Validate user has admin permissions using access token
    console.log('🔄 Admin: Validating admin permissions...');
    const userinfo = await getUserInfo(tokens.accessToken);
    
    // Check for admin permissions using structured permission system
    const hasAdminAccess = userinfo.permissions && userinfo.permissions.some((permission: string) => 
      permission === 'admin:*:*' || permission.startsWith('admin:')
    );
    
    if (!hasAdminAccess) {
      console.error('❌ Admin: User lacks admin permissions', {
        email: userinfo.email,
        permissions: userinfo.permissions
      });
      const loginUrl = new URL('/access-denied', request.url);
      loginUrl.searchParams.set('reason', 'insufficient_admin_permissions');
      return NextResponse.redirect(loginUrl);
    }

    console.log('✅ Admin: User validated with admin permissions:', {
      email: userinfo.email,
      adminPermissions: userinfo.permissions.filter((p: string) => p.startsWith('admin:'))
    });

    // Get callback URL from cookies or default to dashboard
    const callbackUrl = storedCallbackUrl || '/';
    
    console.log('🔄 Redirecting to callback URL:', callbackUrl);
    const redirectUrl = new URL(callbackUrl, request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // OIDC Migration: Set OIDC tokens as HttpOnly cookies
    console.log('🔧 Admin: Setting OIDC tokens as HttpOnly cookies...');
    
    // Set access token (primary authentication token)
    response.cookies.set('access_token', tokens.accessToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: tokens.expiresIn || 3600, // Use token expiry or default 1 hour
      path: '/'
    });
    console.log('✅ OIDC access_token cookie set');
    
    // Set ID token (user identity information)
    if (tokens.idToken) {
      response.cookies.set('id_token', tokens.idToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: tokens.expiresIn || 3600,
        path: '/'
      });
      console.log('✅ OIDC id_token cookie set');
    }
    
    // Set refresh token (for token renewal)
    if (tokens.refreshToken) {
      response.cookies.set('refresh_token', tokens.refreshToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 30 * 24 * 60 * 60, // 30 days for refresh token
        path: '/'
      });
      console.log('✅ OIDC refresh_token cookie set');
    }
    
    // Clean up OAuth PKCE cookies
    console.log('🔧 Admin: Cleaning up OAuth PKCE cookies...');
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');
    response.cookies.delete('pkce_verifier_backup');
    response.cookies.delete('pkce_state_backup');
    // Also clean up legacy cookie during migration
    response.cookies.delete('epsx_admin_jwt');
    console.log('✅ OAuth PKCE cookies cleaned up');
    
    console.log('✅ OIDC callback completed successfully, redirecting with OIDC cookies');
    console.log('🚨 OIDC CALLBACK ROUTE COMPLETED - END');
    return response;

  } catch (error) {
    console.error('❌ OIDC callback processing error:', error);
    
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
    loginUrl.searchParams.set('error', 'oidc_callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    return NextResponse.redirect(loginUrl);
  }
}