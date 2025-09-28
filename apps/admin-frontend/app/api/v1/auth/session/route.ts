/**
 * Admin Session API Route
 * Handles admin session management using unified client
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3AdminClient } from '@/shared/utils/web3-api-client';
import { hasAdminPermissions } from '@/shared/types/wallet-auth';

export async function GET() {
  try {
    const cookieStore = await cookies();
    const adminSession = cookieStore.get('admin_session')?.value;
    
    if (!adminSession) {
      return NextResponse.json({
        isAuthenticated: false,
        error: 'No admin session found'
      }, { status: 401 });
    }

    // Create Web3 admin client for server-side session verification
    const web3Client = createWeb3AdminClient({ serverSide: true });
    
    // Get session data using typed client
    const sessionData = await web3Client.getSession();
    
    // Verify admin permissions
    const hasAdminPerms = hasAdminPermissions(sessionData.permissions);
    
    if (!hasAdminPerms) {
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Insufficient admin permissions'
      }, { status: 403 });
    }
    
    // Return admin session data
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id,
        permissions: sessionData.permissions,
        tier: sessionData.tier || 'admin',
        has_access: true,
        admin_level: 'admin',
      },
      expiresAt: sessionData.expires_at,
    });
  } catch (error) {
    console.error('Admin session verification error:', error);
    return NextResponse.json({
      isAuthenticated: false,
      error: 'Session verification failed'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    // Create Web3 admin client for server-side logout
    const web3Client = createWeb3AdminClient({ serverSide: true });
    
    // Call backend logout endpoint
    await web3Client.logout();
    
    // Clear admin authentication cookies
    const response = NextResponse.json({ 
      success: true, 
      message: 'Admin session cleared successfully' 
    });
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      maxAge: 0,
      path: '/',
    };
    
    response.cookies.set('access_token', '', cookieOptions);
    response.cookies.set('refresh_token', '', cookieOptions);
    response.cookies.set('admin_session', '', cookieOptions);
    response.cookies.set('wallet_address', '', cookieOptions);

    console.log('✅ Admin: Session cleared successfully');
    return response;
  } catch (error) {
    console.error('❌ Admin: Session clearing error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear admin session' },
      { status: 500 }
    );
  }
}