/**
 * Web3 Session API Route
 * Returns current Web3 authentication session data
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const web3Session = cookieStore.get('web3_session')?.value;

    if (!accessToken || !web3Session) {
      return NextResponse.json(
        { 
          isAuthenticated: false,
          error: 'No active Web3 session' 
        },
        { status: 401 }
      );
    }

    // Create Web3 client with token
    const web3Client = createWeb3FrontendClient({ 
      serverSide: true,
      token: accessToken 
    });
    
    // Get current session from backend
    const sessionData = await web3Client.getSession();
    
    console.log('✅ Frontend: Session retrieved for wallet:', sessionData.wallet_address?.slice(0, 8) + '...');
    
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id,
        permissions: sessionData.permissions || [],
        tier: sessionData.tier || 'basic',
        has_access: true
      },
      expiresAt: sessionData.expires_at
    });

  } catch (error) {
    console.error('❌ Frontend: Session check error:', error);
    
    // Clear invalid session cookies
    const cookieStore = await cookies();
    cookieStore.delete('access_token');
    cookieStore.delete('refresh_token');
    cookieStore.delete('web3_session');
    
    return NextResponse.json(
      { 
        isAuthenticated: false,
        error: 'Invalid session' 
      },
      { status: 401 }
    );
  }
}