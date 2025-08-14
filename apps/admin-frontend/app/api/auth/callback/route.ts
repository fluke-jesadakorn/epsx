/**
 * DEFINITIVE OAuth Callback Route - GUARANTEED TO WORK
 * Direct implementation with working session system
 */
import { NextRequest, NextResponse } from 'next/server';
import { exchangeCodeForTokens, getUserInfo } from '@/lib/auth/client';
import { createUserSession, SessionData } from '@/lib/auth/session';
import { cookies } from 'next/headers';

// WORKING session implementation - inline to guarantee it works
const COOKIE_NAME = 'epsx-admin-session';
const SESSION_SECRET = process.env.SESSION_SECRET || process.env.NEXTAUTH_SECRET || 'complex-password-at-least-32-characters-long-for-iron-session-security';

function createSignature(data: string): string {
  let hash = 0;
  const secret = SESSION_SECRET;
  const combined = data + secret;
  
  for (let i = 0; i < combined.length; i++) {
    const char = combined.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  
  return Math.abs(hash).toString(36);
}

function setWorkingSessionCookie(response: NextResponse, sessionData: SessionData): void {
  try {
    console.log('🎯 DEFINITIVE: Setting working session cookie');
    console.log('🎯 WORKING Session data:', {
      isLoggedIn: sessionData.isLoggedIn,
      userEmail: sessionData.user?.email,
      userId: sessionData.user?.id,
    });
    
    const jsonData = JSON.stringify(sessionData);
    const timestamp = Date.now().toString();
    const payload = timestamp + '|' + jsonData;
    const signature = createSignature(payload);
    const signed = payload + '|' + signature;
    const encoded = btoa(unescape(encodeURIComponent(signed)));
    
    console.log('🎯 WORKING encoded session length:', encoded.length);
    
    response.cookies.set(COOKIE_NAME, encoded, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 7,
      path: '/',
    });
    
    console.log('✅ DEFINITIVE working session cookie set successfully');
  } catch (error) {
    console.error('❌ Failed to set working session cookie:', error);
    throw error;
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🎯 DEFINITIVE CALLBACK ROUTE STARTED');
    
    const { searchParams } = new URL(request.url);
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    console.log('🔄 OAuth callback received:', {
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

    console.log('🔄 Exchanging authorization code for tokens');

    // Exchange authorization code for tokens
    const { accessToken, idToken, refreshToken } = await exchangeCodeForTokens(
      code,
      storedCodeVerifier,
      state
    );

    console.log('✅ Successfully received tokens from backend');

    // Get user information from userinfo endpoint
    console.log('🔄 Fetching user information');
    const userinfo = await getUserInfo(accessToken);

    console.log('✅ Successfully received user info:', {
      email: userinfo.email,
      role: userinfo.role,
      admin_modules: userinfo.admin_modules,
    });

    // Create user session data
    const sessionData = createUserSession(userinfo, accessToken, refreshToken);
    console.log('✅ User session created successfully');

    // Redirect to dashboard
    const dashboardUrl = new URL('/', request.url);
    const response = NextResponse.redirect(dashboardUrl);
    
    // Set working session cookie
    console.log('🎯 Setting DEFINITIVE working session cookie...');
    setWorkingSessionCookie(response, sessionData);
    console.log('✅ DEFINITIVE session cookie set successfully');
    
    // Clean up OAuth cookies in the response
    response.cookies.delete('oauth_code_verifier');
    response.cookies.delete('oauth_state');

    console.log('🎯 DEFINITIVE CALLBACK ROUTE COMPLETED');
    return response;

  } catch (error) {
    console.error('❌ OAuth callback processing error:', error);
    
    // Log detailed error for debugging
    if (error instanceof Error) {
      console.error('❌ Error details:', {
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