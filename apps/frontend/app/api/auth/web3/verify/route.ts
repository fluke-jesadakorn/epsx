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
        { error: 'Missing required fields' },
        { status: 400 }
      );
    }

    // Forward to backend Web3 verify endpoint
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/verify`, {
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
      const errorData = await response.json().catch(() => ({ error: 'Verification failed' }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    
    // Set OIDC tokens as httpOnly cookies if provided by backend
    if (data.access_token) {
      const cookieStore = await cookies();
      
      // Set access token
      cookieStore.set('access_token', data.access_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60, // 1 hour
        path: '/',
      });

      // Set refresh token if provided
      if (data.refresh_token) {
        cookieStore.set('refresh_token', data.refresh_token, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 7, // 7 days
          path: '/',
        });
      }

      // Set ID token if provided
      if (data.id_token) {
        cookieStore.set('id_token', data.id_token, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60, // 1 hour
          path: '/',
        });
      }
    }

    // Return success response without sensitive tokens
    return NextResponse.json({
      success: true,
      wallet_address: data.wallet_address,
      user_id: data.user_id,
      email: data.email,
      permissions: data.permissions || [],
    });

  } catch (error) {
    console.error('Web3 verify API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}