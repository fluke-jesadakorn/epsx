import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { generateCodeChallenge, generateCodeVerifier, generateState } from '../../../../../../shared/auth/pkce';
import { getBackendUrl, getAdminUrl, callbackUrls } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Initiating new OAuth flow...');
    
    // Clear any existing OAuth cookies first
    const cookieStore = await cookies();
    const response = NextResponse.json({ success: true });
    
    // Clear existing OAuth cookies
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');
    response.cookies.delete('oauth_callback_url');
    response.cookies.delete('pkce_verifier_backup');
    response.cookies.delete('pkce_state_backup');
    
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
    
    // Set PKCE cookies
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: 900, // 15 minutes
      path: '/',
    };

    response.cookies.set('oauth_code_verifier', codeVerifier, cookieOptions);
    response.cookies.set('oauth_state', state, cookieOptions);
    response.cookies.set('oauth_callback_url', callbackUrl, cookieOptions);

    // Set backup cookies
    response.cookies.set('pkce_verifier_backup', codeVerifier, {
      ...cookieOptions,
      httpOnly: false,
    });
    response.cookies.set('pkce_state_backup', state, {
      ...cookieOptions,
      httpOnly: false,
    });
    
    console.log('✅ Admin: Fresh OAuth flow initiated with new PKCE parameters');
    
    // Return the authorization URL
    return NextResponse.json({
      success: true,
      authorizationUrl: loginUrl.toString()
    });
    
  } catch (error) {
    console.error('❌ Failed to initiate OAuth flow:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to initiate OAuth flow' },
      { status: 500 }
    );
  }
}