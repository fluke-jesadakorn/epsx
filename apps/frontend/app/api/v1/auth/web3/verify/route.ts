/**
 * Web3 Verification API Route
 * Verifies SIWE signatures and establishes Web3 session using unified client
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, signature, nonce, message } = body;

    if (!wallet_address || !signature || !nonce || !message) {
      return NextResponse.json(
        { error: 'Missing required fields for verification' },
        { status: 400 }
      );
    }

    // Create Web3 client for server-side verification
    const web3Client = createWeb3FrontendClient({ serverSide: true });
    
    // Verify signature using typed client
    const authData = await web3Client.verifySignature({
      wallet_address,
      signature,
      nonce,
      message
    });
    // Set authentication cookies
    if (authData.access_token) {
      const cookieStore = await cookies();
      
      const cookieOptions = {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        path: '/',
      };
      
      // Set access token (Bearer token for backend API)
      cookieStore.set('access_token', authData.access_token, {
        ...cookieOptions,
        maxAge: 60 * 60 * 24, // 24 hours
      });

      // Set refresh token if provided
      if (authData.refresh_token) {
        cookieStore.set('refresh_token', authData.refresh_token, {
          ...cookieOptions,
          maxAge: 60 * 60 * 24 * 7, // 7 days
        });
      }

      // Set Web3 session marker
      cookieStore.set('web3_session', '1', {
        ...cookieOptions,
        maxAge: 60 * 60 * 24, // 24 hours
      });
    }

    console.log('✅ Frontend: Verification successful for wallet:', wallet_address.slice(0, 8) + '...');

    // Return user data
    return NextResponse.json({
      success: true,
      wallet_address: authData.wallet_address,
      user_id: authData.user_id,
      permissions: authData.permissions || [],
      tier: authData.tier || 'basic',
      expires_at: authData.expires_at
    });

  } catch (error) {
    console.error('❌ Frontend: Web3 verify error:', error);
    return NextResponse.json(
      { error: 'Authentication verification failed' },
      { status: 500 }
    );
  }
}