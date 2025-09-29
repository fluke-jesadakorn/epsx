import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST() {
  try {
    const cookieStore = await cookies();
    
    // Clear all Web3 session cookies
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: 0, // Expire immediately
      path: '/'
    };

    cookieStore.set('wallet_address', '', cookieOptions);
    cookieStore.set('wallet_signature', '', cookieOptions);
    cookieStore.set('wallet_message', '', cookieOptions);
    cookieStore.set('wallet_nonce', '', cookieOptions);
    cookieStore.set('wallet_chain_id', '', cookieOptions);
    cookieStore.set('wallet_expires_at', '', cookieOptions);

    console.log('✅ Web3 session cookies cleared');

    return NextResponse.json({ 
      success: true, 
      message: 'Session cleared successfully' 
    });

  } catch (error) {
    console.error('❌ Failed to clear Web3 session:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear session data' },
      { status: 500 }
    );
  }
}