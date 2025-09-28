/**
 * Current User API Route
 * Gets current authenticated user information
 * Proxies to backend session endpoint
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET() {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return NextResponse.json({
        user: null,
        error: 'Not authenticated'
      }, { status: 401 });
    }

    // Get current user from backend session
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      return NextResponse.json({
        user: null,
        error: 'Authentication failed'
      }, { status: 401 });
    }

    const sessionData = await response.json();
    
    return NextResponse.json({
      user: {
        wallet_address: sessionData.wallet_address,
        user_id: sessionData.user_id,
        permissions: sessionData.permissions || [],
        tier: sessionData.tier || 'basic',
        has_access: sessionData.has_access || false,
      }
    });
  } catch (error) {
    console.error('Current user fetch error:', error);
    return NextResponse.json({
      user: null,
      error: 'Failed to fetch user data'
    }, { status: 500 });
  }
}