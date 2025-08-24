import { NextRequest, NextResponse } from 'next/server';

// PKCE helper functions
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(digest));
}

function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

export async function POST(request: NextRequest) {
  try {
    const { redirectTo } = await request.json();
    
    // Generate PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateRandomString(32);
    
    console.log('🔄 Admin: Generating PKCE parameters for OAuth authorization...');
    console.log('✅ Admin: Code verifier generated successfully');
    console.log('✅ Admin: Code challenge generated successfully');
    console.log('✅ Admin: State parameter generated successfully');
    
    // Use environment-aware URLs
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const adminUrl = process.env.NEXT_PUBLIC_APP_URL || process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
    
    // Build authorization URL
    const authorizationEndpoint = `${backendUrl}/oauth/authorize`;
    const clientId = 'epsx-admin';
    const redirectUri = `${adminUrl}/api/auth/callback/epsx-backend`;
    const scope = 'openid profile email admin_modules';
    
    console.log('🔧 Admin: OAuth configuration:', {
      authorizationEndpoint,
      clientId,
      redirectUri,
      scope
    });
    
    const params = new URLSearchParams({
      response_type: 'code',
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: scope,
      state: state,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    });
    
    const authorizationUrl = `${authorizationEndpoint}?${params.toString()}`;
    
    console.log('✅ Admin: Authorization URL generated successfully:', authorizationUrl);
    
    // Store PKCE parameters in session/cookies for callback verification
    const response = NextResponse.json({
      success: true,
      authorizationUrl,
      message: 'PKCE parameters generated successfully'
    });
    
    // Set secure cookies for PKCE verification
    response.cookies.set('oauth_code_verifier', codeVerifier, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 300 // 5 minutes
    });
    
    response.cookies.set('oauth_state', state, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 300 // 5 minutes
    });
    
    response.cookies.set('oauth_redirect_to', redirectTo, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 300 // 5 minutes
    });
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin: OAuth initiation failed:', error);
    return NextResponse.json({
      success: false,
      message: 'OAuth initiation failed',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, { status: 500 });
  }
}