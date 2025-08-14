/**
 * OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
// Removed legacy imports - using simplified OAuth flow
import { signJWT, createJWTClaims, createCookieManager } from '@epsx/auth-shared';
import { cookies } from 'next/headers';

// Simple userinfo fetcher for our simplified OAuth flow
async function fetchUserInfo(accessToken: string) {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
  const response = await fetch(`${apiUrl}/oauth/userinfo`, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`UserInfo fetch failed: ${response.status} ${response.statusText}`);
  }

  return await response.json();
}

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

    console.log('🔄 Using authorization code as access token (simplified flow)');

    // In our simplified implementation, the authorization code IS the access token
    const accessToken = code;

    console.log('✅ Using code as access token for simplified flow');

    // Get user information from userinfo endpoint
    console.log('🔄 Fetching user information from EPSX backend');
    const userinfo = await fetchUserInfo(accessToken);

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
    const callbackUrl = cookieStore.get('oauth_callback_url')?.value || '/';
    
    console.log('🔄 Redirecting to callback URL:', callbackUrl);
    const redirectUrl = new URL(callbackUrl, request.url);
    const response = NextResponse.redirect(redirectUrl);
    
    // Set JWT cookie using new cookie manager
    console.log('🔧 Admin: Setting JWT cookie for redirect...');
    try {
      const cookieManager = createCookieManager('admin');
      cookieManager.setAccessTokenCookie(response, jwtToken);
      console.log('✅ JWT cookie set successfully');
    } catch (sessionError) {
      console.error('❌ JWT cookie error:', sessionError);
      throw sessionError;
    }
    
    // Clean up OAuth cookies in the response
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');
    
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