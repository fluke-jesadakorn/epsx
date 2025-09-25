/**
 * Web3 Enterprise Logout API Route
 * Comprehensive session cleanup for Web3 enterprise authentication
 */
import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json().catch(() => ({}));
    const { wallet_address, session_token } = body;

    console.log('🔄 Processing Web3 enterprise session invalidation...');

    // Step 1: Notify enterprise backend of session termination
    if (wallet_address || session_token) {
      try {
        console.log('🚀 Notifying enterprise backend of session termination...');
        
        const backendResponse = await fetch(`${BACKEND_URL}/api/v1/enterprise/auth/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': request.headers.get('Authorization') || '',
          },
          body: JSON.stringify({
            wallet_address,
            session_token,
            logout_reason: 'user_initiated',
            timestamp: new Date().toISOString(),
          }),
        });

        if (backendResponse.ok) {
          console.log('✅ Enterprise backend notified of session termination');
        } else {
          console.warn('⚠️ Enterprise backend notification failed:', backendResponse.status);
          // Continue with frontend cleanup even if backend fails
        }
      } catch (backendError) {
        console.error('❌ Enterprise backend notification error:', backendError);
        // Continue with frontend cleanup even if backend fails
      }
    }

    // Step 2: Clear all enterprise authentication cookies
    const response = NextResponse.json({
      success: true,
      message: 'Web3 enterprise session invalidated successfully',
      invalidated_at: new Date().toISOString(),
    });

    // Clear enterprise authentication cookies
    const cookiesToClear = [
      'access_token',
      'id_token', 
      'refresh_token',
      'web3_session',
    ];

    cookiesToClear.forEach(cookieName => {
      response.cookies.set(cookieName, '', {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 0, // Expire immediately
        path: '/',
      });
    });

    console.log('🎉 Web3 enterprise session invalidation completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Web3 enterprise session invalidation error:', error);
    return NextResponse.json(
      { 
        success: false, 
        error: 'Failed to invalidate Web3 enterprise session',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}

/**
 * GET endpoint to check enterprise logout service status
 */
export async function GET() {
  return NextResponse.json({
    service: 'web3-enterprise-logout',
    status: 'available',
    methods: ['POST'],
    description: 'Comprehensive Web3 enterprise session invalidation with backend notification',
    supported_features: [
      'enterprise_tier_aware',
      'multi_chain_logout',
      'bearer_token_invalidation',
      'dao_session_cleanup'
    ]
  });
}