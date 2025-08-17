/**
 * Frontend OAuth Callback Route for EPSX Backend
 * Handles OAuth authorization callback and creates user session
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens, getUserInfo } from '@/lib/server/auth';
import { signJWT, createJWTClaims } from '@/lib/auth-utils';
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

    console.log('🔄 Frontend: Backend simplified flow - code IS the access token');

    // In the simplified backend flow, the 'code' parameter is actually the access token
    const accessToken = code;
    console.log('✅ Frontend: Using authorization code as access token (simplified flow)');

    // Get user information from userinfo endpoint
    console.log('🔄 Frontend: Fetching user information from EPSX backend');
    const userinfo = await getUserInfo(accessToken);

    console.log('✅ Frontend: Successfully received user info from EPSX backend:', {
      email: userinfo.email,
      role: userinfo.role,
      permissions: userinfo.permissions,
      subscription_tier: userinfo.subscription_tier,
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
    const callbackUrl = cookieStore.get('oauth_callback_url')?.value || '/dashboard';
    
    // Clean up OAuth cookies and create redirect response
    const response = NextResponse.redirect(new URL(callbackUrl, request.url));
    
    // Set JWT cookie
    console.log('🔧 Frontend: Setting JWT cookie for redirect...');
    response.cookies.set('epsx_frontend_jwt', jwtToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 24 * 60 * 60, // 24 hours
      path: '/'
    });
    
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