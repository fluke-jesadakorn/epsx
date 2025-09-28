/**
 * Admin Web3 Session API Route
 * Returns current admin Web3 authentication session data
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3AdminClient } from '@/shared/utils/web3-api-client';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const adminSession = cookieStore.get('admin_session')?.value;

    if (!accessToken || !adminSession) {
      return NextResponse.json(
        { 
          isAuthenticated: false,
          error: 'No active admin Web3 session' 
        },
        { status: 401 }
      );
    }

    // Create Web3 admin client with token
    const web3Client = createWeb3AdminClient({ 
      serverSide: true,
      token: accessToken 
    });
    
    // Get current session from backend
    const sessionData = await web3Client.getSession();
    
    console.log('✅ Admin: Session retrieved for wallet:', sessionData.wallet_address?.slice(0, 8) + '...');
    
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id,
        permissions: sessionData.permissions || [],
        admin_level: 'admin',
        has_access: true
      },
      expiresAt: sessionData.expires_at
    });

  } catch (error) {
    console.error('❌ Admin: Session check error:', error);
    
    // Clear invalid session cookies
    const cookieStore = await cookies();
    cookieStore.delete('access_token');
    cookieStore.delete('admin_session');
    cookieStore.delete('wallet_address');
    
    return NextResponse.json(
      { 
        isAuthenticated: false,
        error: 'Invalid admin session' 
      },
      { status: 401 }
    );
  }
}