import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, email } = body;

    if (!wallet_address || !email) {
      return NextResponse.json(
        { error: 'Wallet address and email are required' },
        { status: 400 }
      );
    }

    // Get access token from cookies
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    // Forward to backend Web3 link-wallet endpoint
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/link-wallet`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ wallet_address, email }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to initiate email linking' }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    return NextResponse.json(data);

  } catch (error) {
    console.error('Web3 link email API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}