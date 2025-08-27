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
  const frontendUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
  
  // Get the original URL they were trying to access
  const callbackUrl = request.nextUrl.searchParams.get('callbackUrl') || `${frontendUrl}/analytics`;
  
  const loginUrl = new URL('/oauth/authorize', backendUrl);
  loginUrl.searchParams.set('client_id', 'epsx-frontend');
  loginUrl.searchParams.set('response_type', 'code');
  loginUrl.searchParams.set('scope', 'openid profile email');
  loginUrl.searchParams.set('redirect_uri', `${frontendUrl}/api/auth/callback/epsx-backend`);
  loginUrl.searchParams.set('state', state);
  loginUrl.searchParams.set('code_challenge', codeChallenge);
  loginUrl.searchParams.set('code_challenge_method', 'S256');
  
  console.log('🔄 API Route: Generated PKCE parameters for OAuth');
  console.log('🔄 API Route: Redirecting /login to backend OAuth:', loginUrl.toString());
  
  // Create response and set PKCE cookies
  const response = NextResponse.redirect(loginUrl.toString());
  
  const cookieStore = await cookies();
  
  // Set PKCE cookies
  response.cookies.set('oauth_code_verifier', codeVerifier, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 600, // 10 minutes
    path: '/'
  });
  
  response.cookies.set('oauth_state', state, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 600, // 10 minutes
    path: '/'
  });
  
  response.cookies.set('oauth_callback_url', callbackUrl, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 600, // 10 minutes
    path: '/'
  });
  
  console.log('✅ API Route: PKCE cookies set successfully');
  
  return response;
}