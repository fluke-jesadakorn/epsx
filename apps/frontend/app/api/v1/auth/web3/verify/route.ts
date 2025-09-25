/**
 * Web3 Verification API Route
 * Verifies SIWE signatures and establishes Web3 session
 * Aligns with backend /api/v1/auth/web3/verify
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

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

    // Forward to backend's standard Web3 verify endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ 
        wallet_address, 
        signature, 
        nonce, 
        message 
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Verification failed' 
      }));
      console.warn('❌ Verification failed for wallet:', wallet_address.slice(0, 8) + '...');
      return NextResponse.json(errorData, { status: response.status });
    }

    const authData = await response.json();
    
    // Set Bearer token as httpOnly cookie
    if (authData.access_token) {
      const cookieStore = await cookies();
      
      // Set access token (Bearer token for backend API)
      cookieStore.set('access_token', authData.access_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24, // 24 hours
        path: '/',
      });

      // Set refresh token if provided
      if (authData.refresh_token) {
        cookieStore.set('refresh_token', authData.refresh_token, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 7, // 7 days
          path: '/',
        });
      }

      // Set Web3 session marker
      cookieStore.set('web3_session', '1', {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24, // 24 hours
        path: '/',
      });
    }

    console.log('✅ Verification successful for wallet:', wallet_address.slice(0, 8) + '...');

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
    console.error('❌ Web3 verify API error:', error);
    return NextResponse.json(
      { error: 'Authentication service unavailable' },
      { status: 500 }
    );
  }
}