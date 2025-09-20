import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;
    
    console.log('🔄 Admin: Logging out wallet session:', walletAddress || 'unknown');

    // Notify backend of logout (optional)
    if (walletAddress) {
      try {
        await fetch(`${BACKEND_URL}/api/auth/web3/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Admin-Context': 'true',
          },
          body: JSON.stringify({ 
            wallet_address: walletAddress,
            admin_context: true
          }),
        });
      } catch (error) {
        console.warn('⚠️ Admin: Backend logout notification failed:', error);
        // Continue with client-side logout even if backend fails
      }
    }

    // Clear all wallet session cookies
    const cookieOptions = {
      path: '/',
      maxAge: 0, // Expire immediately
    };

    cookieStore.set('wallet_address', '', cookieOptions);
    cookieStore.set('wallet_nonce', '', cookieOptions);
    cookieStore.set('wallet_signature', '', cookieOptions);
    cookieStore.set('wallet_message', '', cookieOptions);
    cookieStore.set('wallet_expires_at', '', cookieOptions);
    
    // Also clear legacy OIDC tokens
    cookieStore.set('access_token', '', cookieOptions);
    cookieStore.set('id_token', '', cookieOptions);
    cookieStore.set('refresh_token', '', cookieOptions);
    cookieStore.set('admin_jwt_token', '', cookieOptions);
    cookieStore.set('session_token', '', cookieOptions);
    
    console.log('✅ Admin: Wallet session cleared successfully');
    
    return NextResponse.json({
      success: true,
      message: 'Admin wallet session cleared',
      timestamp: Date.now()
    });

  } catch (error) {
    console.error('❌ Admin: Web3 logout API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  // Allow GET for compatibility
  return await POST(request);
}