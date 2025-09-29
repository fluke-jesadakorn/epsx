import { NextRequest, NextResponse } from 'next/server';
import { getServerSession, isAdmin } from '@/lib/auth/server-auth';

export async function POST(request: NextRequest) {
  try {
    console.log('🔍 Web3 auth verification endpoint called');
    
    const body = await request.json();
    const adminContext = body?.admin_context || false;
    
    const session = await getServerSession();
    
    console.log('🔍 Web3 auth verification:', {
      hasSession: !!session,
      userId: session?.user?.id || 'no user_id',
      walletAddress: session?.user?.wallet_address || 'no wallet',
      adminContext,
      permissions: session?.user?.permissions || []
    });

    if (!session?.user) {
      console.log('❌ No active session found');
      return NextResponse.json({
        success: false,
        error: 'No active session'
      }, { status: 401 });
    }

    // For admin context, check if user has admin permissions
    if (adminContext) {
      const userIsAdmin = isAdmin(session.user);
      if (!userIsAdmin) {
        console.log('❌ User does not have admin permissions');
        return NextResponse.json({
          success: false,
          error: 'Admin permissions required'
        }, { status: 403 });
      }
    }

    console.log('✅ Authentication verified successfully');
    return NextResponse.json({
      success: true,
      authenticated: true,
      wallet_address: session.user.wallet_address || session.user.sub,
      user_id: session.user.id,
      permissions: session.user.permissions || [],
      is_admin: isAdmin(session.user),
      expires: session.expires
    });

  } catch (error) {
    console.error('❌ Web3 auth verification error:', error);
    return NextResponse.json({
      success: false,
      error: error instanceof Error ? error.message : 'Verification failed'
    }, { status: 500 });
  }
}

export async function GET(request: NextRequest) {
  // Allow GET requests for simple auth checks
  return POST(request);
}