/**
 * Session API Route
 * Handles Web3 session management using unified API client
 */
import { NextRequest, NextResponse } from 'next/server';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function GET() {
  try {
    // Create Web3 client for server-side use
    const web3Client = createWeb3FrontendClient({ serverSide: true });
    
    // Get session data using typed client
    const sessionData = await web3Client.getSession();
    
    // Return session data in standardized format
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id,
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
    // Create Web3 client for server-side logout
    const web3Client = createWeb3FrontendClient({ serverSide: true });
    
    // Call backend logout endpoint
    await web3Client.logout();
    
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

    console.log('✅ Frontend: Session cleared successfully');
    return response;
  } catch (error) {
    console.error('❌ Frontend: Session clearing error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear session' },
      { status: 500 }
    );
  }
}