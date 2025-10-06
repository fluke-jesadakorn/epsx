import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

/**
 *
 * @param request
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { walletAddress, signature, message, nonce, chainId } = body;

    if (!walletAddress || !signature || !message || !nonce || !chainId) {
      return NextResponse.json(
        { success: false, error: 'Missing required session data' },
        { status: 400 }
      );
    }

    const cookieStore = await cookies();
    
    // Store Web3 session data in cookies (expires in 24 hours)
    const expiresAt = Date.now() + (24 * 60 * 60 * 1000);
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: 24 * 60 * 60, // 24 hours
      path: '/'
    };

    // Set all required cookies for server-side session validation
    cookieStore.set('wallet_address', walletAddress, cookieOptions);
    cookieStore.set('wallet_signature', signature, cookieOptions);
    cookieStore.set('wallet_message', message, cookieOptions);
    cookieStore.set('wallet_nonce', nonce, cookieOptions);
    cookieStore.set('wallet_chain_id', chainId.toString(), cookieOptions);
    cookieStore.set('wallet_expires_at', expiresAt.toString(), cookieOptions);

    return NextResponse.json({ 
      success: true, 
      message: 'Session stored successfully',
      expiresAt 
    });

  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('❌ Failed to store Web3 session:', _error);
    return NextResponse.json(
      { success: false, error: 'Failed to store session data' },
      { status: 500 }
    );
  }
}