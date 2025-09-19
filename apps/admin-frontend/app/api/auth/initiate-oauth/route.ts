import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { generateCodeChallenge, generateCodeVerifier, generateState } from '../../../../../../shared/auth/pkce';
import { getBackendUrl, getAdminUrl, callbackUrls } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Initiating new OAuth flow...');
    
    // Clear any existing OAuth cookies first
    const cookieStore = await cookies();
    
    // Generate fresh PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateState();
    
    const backendUrl = getBackendUrl('server');
    const adminUrl = getAdminUrl('server');
    
    // Get redirect URL from request body
    const body = await request.json().catch(() => ({}));
    const callbackUrl = body.redirectTo || `${adminUrl}/`;
    
    const loginUrl = new URL('/oauth/authorize', backendUrl);
    loginUrl.searchParams.set('client_id', 'epsx-admin');
    loginUrl.searchParams.set('response_type', 'code');
    loginUrl.searchParams.set('scope', 'openid profile email permissions');
    loginUrl.searchParams.set('redirect_uri', callbackUrls.admin('server'));
    loginUrl.searchParams.set('state', state);
    loginUrl.searchParams.set('code_challenge', codeChallenge);
    loginUrl.searchParams.set('code_challenge_method', 'S256');
    
    // Set PKCE cookies with enhanced persistence
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: 1800, // 30 minutes (extended from 15)
      path: '/',
    };
    
    console.log('✅ Admin: Fresh OAuth flow initiated with new PKCE parameters');
    
    // Create final response with all cookies attached
    const finalResponse = NextResponse.json({
      success: true,
      authorizationUrl: loginUrl.toString(),
      // Include PKCE parameters for client-side backup in development
      debug: process.env.NODE_ENV === 'development' ? {
        codeVerifier,
        state,
        callbackUrl
      } : undefined
    });

    // Clear existing OAuth cookies on final response
    finalResponse.cookies.delete('oauth_code_verifier');
    finalResponse.cookies.delete('oauth_state');
    finalResponse.cookies.delete('oauth_callback_url');
    finalResponse.cookies.delete('pkce_verifier_backup');
    finalResponse.cookies.delete('pkce_state_backup');

    // Set primary PKCE cookies on final response
    finalResponse.cookies.set('oauth_code_verifier', codeVerifier, cookieOptions);
    finalResponse.cookies.set('oauth_state', state, cookieOptions);
    finalResponse.cookies.set('oauth_callback_url', callbackUrl, cookieOptions);

    // Set backup cookies (accessible to JavaScript for debugging)
    finalResponse.cookies.set('pkce_verifier_backup', codeVerifier, {
      ...cookieOptions,
      httpOnly: false,
    });
    finalResponse.cookies.set('pkce_state_backup', state, {
      ...cookieOptions,
      httpOnly: false,
    });
    
    // Set additional fallback cookies with different names
    finalResponse.cookies.set('admin_oauth_verifier', codeVerifier, cookieOptions);
    finalResponse.cookies.set('admin_oauth_state', state, cookieOptions);
    
    console.log('🍪 Admin: PKCE cookies set on final response:', {
      codeVerifier: `${codeVerifier.slice(0, 10)}...`,
      state: `${state.slice(0, 10)}...`,
      callbackUrl,
      maxAge: cookieOptions.maxAge
    });
    
    return finalResponse;
    
  } catch (error) {
    console.error('❌ Failed to initiate OAuth flow:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to initiate OAuth flow' },
      { status: 500 }
    );
  }
}