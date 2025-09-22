import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

/**
 * Web3 Session Invalidation Endpoint
 * 
 * Provides comprehensive session cleanup specifically for Web3 authentication flows.
 * This endpoint ensures both frontend and backend sessions are properly invalidated.
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json().catch(() => ({}));
    const { wallet_address, session_token } = body;

    console.log('🔄 Processing Web3 session invalidation request...');

    // Step 1: Notify backend of Web3 session termination (if we have session info)
    if (wallet_address || session_token) {
      try {
        console.log('🚀 Notifying backend of Web3 session termination...');
        
        const backendResponse = await fetch(`${BACKEND_URL}/api/auth/web3/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': request.headers.get('Authorization') || '',
          },
          body: JSON.stringify({
            wallet_address,
            session_token,
            timestamp: new Date().toISOString(),
          }),
        });

        if (backendResponse.ok) {
          console.log('✅ Backend notified of Web3 session termination');
        } else {
          console.warn('⚠️ Backend notification failed:', backendResponse.status);
          // Continue with frontend cleanup even if backend fails
        }
      } catch (backendError) {
        console.error('❌ Backend notification error:', backendError);
        // Continue with frontend cleanup even if backend fails
      }
    }

    // Step 2: Clear all authentication cookies
    const response = NextResponse.json({
      success: true,
      message: 'Web3 session invalidated successfully',
      invalidated_at: new Date().toISOString(),
    });

    // Clear all possible authentication cookies
    const cookiesToClear = [
      'access_token',
      'id_token', 
      'refresh_token',
      'oidc_session',
      'epsx_frontend_jwt',
      'next-auth.session-token',
      'next-auth.csrf-token',
      'web3_auth_session',
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

    console.log('🎉 Web3 session invalidation completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Web3 session invalidation error:', error);
    return NextResponse.json(
      { 
        success: false, 
        error: 'Failed to invalidate Web3 session',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}

/**
 * GET endpoint to check invalidation status
 */
export async function GET() {
  return NextResponse.json({
    service: 'web3-session-invalidation',
    status: 'available',
    methods: ['POST'],
    description: 'Comprehensive Web3 session invalidation with backend notification',
  });
}