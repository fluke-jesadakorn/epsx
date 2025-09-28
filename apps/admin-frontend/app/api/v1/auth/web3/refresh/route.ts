/**
 * Admin Web3 Session Refresh API Route
 * Refreshes admin Web3 authentication session and updates tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3AdminClient } from '@/shared/utils/web3-api-client';

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

    // Create Web3 admin client with refresh token
    const web3Client = createWeb3AdminClient({ 
      serverSide: true,
      token: refreshToken 
    });
    
    // Refresh session through backend
    const refreshedSession = await web3Client.refreshSession();
    
    if (!refreshedSession) {
      throw new Error('Admin session refresh failed');
    }

    // Update cookies with new tokens
    const expiresIn = 3600; // 1 hour for admin sessions
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/',
      maxAge: expiresIn,
    };
    
    // Set new access token
    if (refreshedSession.access_token) {
      cookieStore.set('access_token', refreshedSession.access_token, cookieOptions);
    }

    // Set new refresh token if provided
    if (refreshedSession.refresh_token) {
      cookieStore.set('refresh_token', refreshedSession.refresh_token, {
        ...cookieOptions,
        maxAge: 60 * 60 * 24 * 3, // 3 days for admin refresh
      });
    }

    // Update admin session marker
    cookieStore.set('admin_session', '1', cookieOptions);

    console.log('✅ Admin: Session refreshed for wallet:', refreshedSession.wallet_address?.slice(0, 8) + '...');
    
    return NextResponse.json({
      success: true,
      wallet_address: refreshedSession.wallet_address,
      user_id: refreshedSession.user_id,
      permissions: refreshedSession.permissions || [],
      admin_level: 'admin',
      expires_at: refreshedSession.expiresAt
    });

  } catch (error) {
    console.error('❌ Admin: Session refresh error:', error);
    
    // Clear invalid tokens
    const cookieStore = await cookies();
    cookieStore.delete('access_token');
    cookieStore.delete('refresh_token');
    cookieStore.delete('admin_session');
    cookieStore.delete('wallet_address');
    
    return NextResponse.json(
      { error: 'Admin session refresh failed' },
      { status: 401 }
    );
  }
}