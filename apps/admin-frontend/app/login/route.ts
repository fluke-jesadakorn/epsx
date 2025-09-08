import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { generateCodeChallenge, generateCodeVerifier, generateState } from '@/lib/pkce';

export async function GET(request: NextRequest) {
  // Generate PKCE parameters
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = await generateCodeChallenge(codeVerifier);
  const state = generateState();
  
  const backendUrl = process.env.NEXT_PUBLIC_API_URL || 
                    process.env.NEXT_PUBLIC_BACKEND_URL || 
                    'http://localhost:8080';
  const adminUrl = process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
  
  // Get the original URL they were trying to access
  const callbackUrl = request.nextUrl.searchParams.get('redirectTo') || `${adminUrl}/`;
  
  const loginUrl = new URL('/oauth/authorize', backendUrl);
  loginUrl.searchParams.set('client_id', 'epsx-admin');
  loginUrl.searchParams.set('response_type', 'code');
  loginUrl.searchParams.set('scope', 'openid profile email permissions');
  loginUrl.searchParams.set('redirect_uri', `${adminUrl}/api/auth/callback/epsx-backend`);
  loginUrl.searchParams.set('state', state);
  loginUrl.searchParams.set('code_challenge', codeChallenge);
  loginUrl.searchParams.set('code_challenge_method', 'S256');
  
  console.log('🔄 Admin Route: Generated PKCE parameters for OAuth');
  console.log('🔄 Admin Route: Redirecting /login to backend OAuth:', loginUrl.toString());
  
  // Create response and set PKCE cookies
  const response = NextResponse.redirect(loginUrl.toString());
  
  const cookieStore = await cookies();
  
  // Set PKCE cookies with more permissive settings for development
  const cookieOptions = {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    maxAge: 900, // 15 minutes - longer timeout for OAuth flow
    path: '/',
    // For development, don't restrict domain
    ...(process.env.NODE_ENV === 'development' ? {} : {})
  };

  response.cookies.set('oauth_code_verifier', codeVerifier, cookieOptions);
  response.cookies.set('oauth_state', state, cookieOptions);
  response.cookies.set('oauth_callback_url', callbackUrl, cookieOptions);

  // Also set backup cookies in case primary ones are lost
  response.cookies.set('pkce_verifier_backup', codeVerifier, {
    ...cookieOptions,
    httpOnly: false, // Make accessible to client-side as backup
  });
  response.cookies.set('pkce_state_backup', state, {
    ...cookieOptions,
    httpOnly: false, // Make accessible to client-side as backup
  });
  
  console.log('✅ Admin Route: PKCE cookies set successfully');
  
  return response;
}