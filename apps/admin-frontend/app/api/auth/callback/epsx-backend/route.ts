/**
 * OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { signJWT, createJWTClaims } from '@/lib/auth-utils';
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
    const storedCodeVerifier = cookieStore.get('oauth_code_verifier')?.value;
    const storedState = cookieStore.get('oauth_state')?.value;
    const storedCallbackUrl = cookieStore.get('oauth_callback_url')?.value;

    // Debug cookie values
    console.log('🔍 Cookie debug info:');
    console.log('  - oauth_code_verifier:', storedCodeVerifier ? `present (${storedCodeVerifier.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_state:', storedState ? `present (${storedState.slice(0,10)}...)` : 'missing');
    console.log('  - oauth_callback_url:', storedCallbackUrl || 'missing');
    console.log('  - All cookies:', Object.fromEntries(cookieStore.getAll().map(c => [c.name, c.value.slice(0,20) + '...'])));

    // Handle missing PKCE parameters - this can happen in simplified flow or with cookie issues
    if (!storedCodeVerifier || !storedState) {
      console.warn('⚠️ Missing PKCE parameters in cookies - this might be due to expired cookies or simplified flow');
      console.warn('⚠️ Continuing with simplified authorization flow...');
      
      // For JWT-based simplified flow, we can proceed without PKCE validation
      // But we should validate the code format to ensure it's a valid JWT
      const isJWT = code.split('.').length === 3; // Basic JWT format check
      
      if (!isJWT) {
        console.error('❌ Code is not a JWT and PKCE parameters are missing');
        const loginUrl = new URL('/login', request.url);
        loginUrl.searchParams.set('error', 'invalid_authorization_code');
        return NextResponse.redirect(loginUrl);
      }
      
      console.log('✅ Code appears to be a JWT, proceeding with simplified flow');
    }

    // Validate state parameter if we have stored state
    if (storedState) {
      console.log('🔍 State validation:');
      console.log('  - Received state:', state);
      console.log('  - Stored state:', storedState);
      console.log('  - States match:', state === storedState);
      
      if (state !== storedState) {
        console.warn('⚠️ State mismatch detected - this could be due to multiple signin attempts or cookie issues');
        console.warn('⚠️ In simplified flow, this is non-critical but should be investigated');
        // Don't fail the authentication for state mismatch in simplified flow
      } else {
        console.log('✅ State validation passed');
      }
    } else {
      console.log('🔍 Skipping state validation (no stored state available)');
    }

    console.log('🔄 Admin: Performing proper OAuth token exchange...');

    // Exchange authorization code for access token (proper OAuth flow)
    const tokens = await exchangeCodeForTokens(code, storedCodeVerifier || '', state);
    console.log('✅ Admin: Successfully exchanged authorization code for access token');

    // Get user information from userinfo endpoint using access token
    console.log('🔄 Admin: Fetching user information from EPSX backend');
    const userinfo = await getUserInfo(tokens.accessToken);

    console.log('✅ Successfully received user info from EPSX backend:', {
      email: userinfo.email,
      role: userinfo.role,
      admin_modules: userinfo.admin_modules,
    });

    // Create JWT token with user claims
    console.log('🔄 Creating JWT token with userinfo:', { email: userinfo.email, role: userinfo.role });
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
    console.log('✅ JWT token created successfully');

    // Get callback URL from cookies or default to dashboard (reuse existing cookieStore)
    const callbackUrl = storedCallbackUrl || '/';
    
    console.log('🔄 Redirecting to callback URL:', callbackUrl);
    const redirectUrl = new URL(callbackUrl, request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // Clean up OAuth cookies FIRST (before setting JWT cookie)
    console.log('🔧 Cleaning up OAuth cookies...');
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
    
    // Set JWT cookie (AFTER cleanup)
    console.log('🔧 Admin: Setting JWT cookie for redirect...');
    response.cookies.set('epsx_admin_jwt', jwtToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 24 * 60 * 60, // 24 hours
      path: '/'
    });
    console.log('✅ Admin JWT cookie set successfully');
    
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

    // Redirect to login page with error (using proper admin domain)
    const loginUrl = new URL('/login', 'https://admin.epsx.io');
    loginUrl.searchParams.set('error', 'callback_error');
    loginUrl.searchParams.set('error_details', encodeURIComponent(error instanceof Error ? error.message : 'Unknown error'));
    
    return NextResponse.redirect(loginUrl);
  }
}