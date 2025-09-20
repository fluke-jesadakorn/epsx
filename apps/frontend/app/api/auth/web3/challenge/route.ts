import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address } = body;

    if (!wallet_address) {
      return NextResponse.json(
        { error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    // Forward to backend Web3 challenge endpoint
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Challenge generation failed' }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    return NextResponse.json(data);

  } catch (error) {
    console.error('Web3 challenge API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}