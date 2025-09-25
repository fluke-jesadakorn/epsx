/**
 * Web3 Challenge API Route
 * Generates SIWE challenges for wallet authentication
 * Aligns with backend /api/v1/auth/web3/challenge
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
        { error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    // Forward to backend's standard Web3 challenge endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ wallet_address }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Challenge generation failed' 
      }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Challenge generated for wallet:', wallet_address.slice(0, 8) + '...');
    
    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Web3 challenge API error:', error);
    return NextResponse.json(
      { error: 'Authentication service unavailable' },
      { status: 500 }
    );
  }
}