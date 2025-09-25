/**
 * Logout Route for Frontend
 * Handles Web3 session cleanup and wallet disconnection
 */

import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { getBackendUrl, URLContext } from '@/lib/server-shared';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Initiating logout process...');
    
    const cookieStore = await cookies();
    
    // Check Web3 authentication
    const walletAddress = cookieStore.get('wallet_address')?.value;
    const web3Session = cookieStore.get('web3_session')?.value;
    const isWeb3Auth = !!walletAddress && !!web3Session;
    
    console.log('🔍 Frontend: Web3 logout detection:', {
      isWeb3Auth,
      hasWalletAddress: !!walletAddress,
      hasWeb3Session: !!web3Session
    });

    // Notify backend of Web3 logout if we have a session
    if (isWeb3Auth) {
      try {
        const backendLogoutUrl = `${getBackendUrl(URLContext.SERVER)}/api/auth/web3/logout`;
        
        console.log('🔄 Frontend: Notifying Web3 backend logout:', backendLogoutUrl);
        
        await fetch(backendLogoutUrl, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${web3Session}`,
            'X-Wallet-Address': walletAddress
          },
          credentials: 'include'
        });
        
        console.log('✅ Frontend: Web3 backend logout notification completed');
      } catch (backendError) {
        console.error('⚠️ Frontend: Web3 backend logout notification failed (continuing with local cleanup):', backendError);
        // Continue with local cleanup even if backend fails
      }
    }

    // Clear all Web3 authentication-related cookies
    const cookiesToClear = [
      // Web3 authentication
      'web3_session',
      'wallet_address',
      'wallet_nonce',
      'wallet_signature',
      'wallet_message',
      'wallet_expires_at',
      
      // Legacy cookies for cleanup
      'session_token',
      'epsx_frontend_jwt'
    ];

    const response = NextResponse.json({
      success: true,
      message: 'Web3 logout completed successfully',
      clearedCookies: cookiesToClear.length,
      authMethod: isWeb3Auth ? 'web3' : 'none'
    });

    // Clear all cookies
    cookiesToClear.forEach(cookieName => {
      const existingCookie = cookieStore.get(cookieName);
      if (existingCookie) {
        response.cookies.delete(cookieName);
        console.log(`🗑️ Frontend: Cleared cookie: ${cookieName}`);
      }
    });

    // Also clear cookies with different paths
    cookiesToClear.forEach(cookieName => {
      response.cookies.set(cookieName, '', {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 0,
        path: '/',
        expires: new Date(0)
      });
    });

    console.log('✅ Frontend: Complete logout successful - all cookies cleared');

    return response;

  } catch (error) {
    console.error('❌ Frontend: Logout failed:', error);
    
    return NextResponse.json({
      success: false,
      error: 'logout_failed',
      message: error instanceof Error ? error.message : 'Logout process failed'
    }, { status: 500 });
  }
}

export async function GET() {
  return NextResponse.json({
    error: 'method_not_allowed',
    message: 'Logout endpoint requires POST method',
    availableMethods: ['POST']
  }, { status: 405 });
}