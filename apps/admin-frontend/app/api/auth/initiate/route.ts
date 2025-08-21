/**
 * Admin Frontend OAuth Initiation Route for EPSX Backend
 * Generates PKCE parameters and provides authorization URL
 */
import { NextRequest, NextResponse } from 'next/server';
import { getAuthorizationUrl } from '@/lib/server/auth';
import { cookies } from 'next/headers';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { redirectTo = '/dashboard' } = body;

    console.log('🔄 Admin: Initiating OAuth flow with PKCE...');

    // Generate authorization URL with PKCE parameters
    const { url, codeVerifier, state } = await getAuthorizationUrl();

    // Store PKCE parameters in httpOnly cookies
    const cookieStore = await cookies();
    
    // Set secure cookies with PKCE parameters
    cookieStore.set('oauth_code_verifier', codeVerifier, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 10 * 60, // 10 minutes (OAuth flow should be quick)
      path: '/'
    });

    cookieStore.set('oauth_state', state, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 10 * 60, // 10 minutes
      path: '/'
    });

    cookieStore.set('oauth_callback_url', redirectTo, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 10 * 60, // 10 minutes
      path: '/'
    });

    console.log('✅ Admin: PKCE parameters stored in cookies successfully');

    return NextResponse.json({ 
      success: true, 
      authorizationUrl: url 
    });

  } catch (error) {
    console.error('❌ Admin: OAuth initiation failed:', error);
    
    return NextResponse.json(
      { 
        success: false, 
        error: 'OAuth initiation failed',
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}