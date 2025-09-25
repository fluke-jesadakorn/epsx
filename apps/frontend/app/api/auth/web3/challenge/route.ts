/**
 * Web3 Enterprise Challenge API Route
 * Generates SIWE challenges for wallet authentication
 */
import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address } = body;

    if (!wallet_address) {
      return NextResponse.json(
        { error: 'Wallet address is required for enterprise authentication' },
        { status: 400 }
      );
    }

    // Forward to enterprise API challenge endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/enterprise/auth/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Enterprise challenge generation failed' 
      }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Enterprise challenge generated for wallet:', wallet_address.slice(0, 8) + '...');
    
    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Web3 enterprise challenge API error:', error);
    return NextResponse.json(
      { error: 'Enterprise authentication service unavailable' },
      { status: 500 }
    );
  }
}