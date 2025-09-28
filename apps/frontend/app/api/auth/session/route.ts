/**
 * Frontend Session API Route
 * Handles Web3 wallet session management via backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET() {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return NextResponse.json({
        isAuthenticated: false,
        error: 'No active session'
      }, { status: 401 });
    }

    // Use the correct backend Web3 session endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      console.error('❌ Frontend: Backend session verification failed:', response.status);
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Invalid session'
      }, { status: 401 });
    }

    const sessionData = await response.json();
    
    // Return session data in frontend-compatible format
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id || sessionData.wallet_address,
        permissions: sessionData.permissions || [],
        tier: sessionData.tier || 'basic',
        has_access: true,
      },
      expiresAt: sessionData.expires_at,
    });
  } catch (error) {
    console.error('❌ Frontend: Session verification error:', error);
    return NextResponse.json({
      isAuthenticated: false,
      error: 'Session verification failed'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    // Notify backend about logout if we have a token
    if (accessToken) {
      try {
        await fetch(`${BACKEND_URL}/api/v1/auth/web3/logout`, {
          method: 'DELETE',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
          },
        });
      } catch (error) {
        console.warn('❌ Frontend: Failed to notify backend about logout:', error);
        // Continue with local logout even if backend fails
      }
    }
    
    // Clear authentication cookies
    const response = NextResponse.json({ 
      success: true, 
      message: 'Session cleared successfully' 
    });
    
    response.cookies.set('access_token', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });
    
    response.cookies.set('refresh_token', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });

    response.cookies.set('web3_session', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });

    console.log('✅ Frontend: Session cookies cleared successfully');
    return response;
  } catch (error) {
    console.error('❌ Frontend: Session clearing error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear session' },
      { status: 500 }
    );
  }
}