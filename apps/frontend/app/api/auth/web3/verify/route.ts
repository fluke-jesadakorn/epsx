/**
 * Web3 Enterprise Verification API Route
 * Verifies SIWE signatures and establishes enterprise session
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
        { error: 'Missing required fields for enterprise verification' },
        { status: 400 }
      );
    }

    // Forward to enterprise API verify endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/enterprise/auth/verify`, {
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
        error: 'Enterprise verification failed' 
      }));
      console.warn('❌ Enterprise verification failed for wallet:', wallet_address.slice(0, 8) + '...');
      return NextResponse.json(errorData, { status: response.status });
    }

    const enterpriseData = await response.json();
    
    // Set enterprise Bearer tokens as httpOnly cookies
    if (enterpriseData.access_token) {
      const cookieStore = await cookies();
      
      // Set access token (Bearer token for enterprise API)
      cookieStore.set('access_token', enterpriseData.access_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24, // 24 hours for enterprise sessions
        path: '/',
      });

      // Set refresh token if provided
      if (enterpriseData.refresh_token) {
        cookieStore.set('refresh_token', enterpriseData.refresh_token, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 30, // 30 days for enterprise refresh
          path: '/',
        });
      }

      // Set enterprise session marker
      cookieStore.set('web3_session', '1', {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24, // 24 hours
        path: '/',
      });
    }

    console.log('✅ Enterprise verification successful for wallet:', wallet_address.slice(0, 8) + '...');

    // Return enterprise-specific response
    return NextResponse.json({
      success: true,
      wallet_address: enterpriseData.wallet_address,
      enterprise_tier: enterpriseData.enterprise_tier || 'Starter',
      permissions: enterpriseData.permissions || [],
      has_api_access: enterpriseData.has_api_access || false,
      verified_tokens_usd: enterpriseData.verified_tokens_usd || 0,
      nft_collections: enterpriseData.nft_collections || [],
      dao_memberships: enterpriseData.dao_memberships || [],
    });

  } catch (error) {
    console.error('❌ Web3 enterprise verify API error:', error);
    return NextResponse.json(
      { error: 'Enterprise authentication service unavailable' },
      { status: 500 }
    );
  }
}