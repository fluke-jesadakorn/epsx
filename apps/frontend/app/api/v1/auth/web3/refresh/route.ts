/**
 * Web3 Session Refresh API Route
 * Refreshes Web3 authentication session and updates tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function POST(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const refreshToken = cookieStore.get('refresh_token')?.value;

    if (!refreshToken) {
      return NextResponse.json(
        { error: 'No refresh token available' },
        { status: 401 }
      );
    }

    // Create Web3 client with refresh token
    const web3Client = createWeb3FrontendClient({ 
      serverSide: true,
      token: refreshToken 
    });
    
    // Refresh session through backend
    const refreshedSession = await web3Client.refreshSession();
    
    if (!refreshedSession) {
      throw new Error('Session refresh failed');
    }

    // Update cookies with new tokens
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/',
    };
    
    // Set new access token
    if (refreshedSession.access_token) {
      cookieStore.set('access_token', refreshedSession.access_token, {
        ...cookieOptions,
        maxAge: 60 * 60 * 24, // 24 hours
      });
    }

    // Set new refresh token if provided
    if (refreshedSession.refresh_token) {
      cookieStore.set('refresh_token', refreshedSession.refresh_token, {
        ...cookieOptions,
        maxAge: 60 * 60 * 24 * 7, // 7 days
      });
    }

    console.log('✅ Frontend: Session refreshed for wallet:', refreshedSession.wallet_address?.slice(0, 8) + '...');
    
    return NextResponse.json({
      success: true,
      wallet_address: refreshedSession.wallet_address,
      user_id: refreshedSession.user_id,
      permissions: refreshedSession.permissions || [],
      tier: refreshedSession.tier_level || 'basic',
      expires_at: refreshedSession.expires_at
    });

  } catch (error) {
    console.error('❌ Frontend: Session refresh error:', error);
    
    // Clear invalid tokens
    const cookieStore = await cookies();
    cookieStore.delete('access_token');
    cookieStore.delete('refresh_token');
    cookieStore.delete('web3_session');
    
    return NextResponse.json(
      { error: 'Session refresh failed' },
      { status: 401 }
    );
  }
}