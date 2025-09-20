import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

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

    // Validate wallet address format
    if (!/^0x[a-fA-F0-9]{40}$/.test(wallet_address)) {
      return NextResponse.json(
        { error: 'Invalid wallet address format' },
        { status: 400 }
      );
    }

    console.log('🔄 Admin: Generating Web3 challenge for wallet:', wallet_address);

    // Forward to backend Web3 challenge endpoint with admin context
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/challenge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Admin-Context': 'true', // Mark as admin authentication request
      },
      body: JSON.stringify({ 
        wallet_address,
        admin_context: true // Request admin-specific challenge
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Challenge generation failed' }));
      console.error('❌ Admin: Backend challenge generation failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    
    console.log('✅ Admin: Web3 challenge generated successfully for:', wallet_address);
    
    return NextResponse.json({
      nonce: data.nonce,
      wallet_address,
      timestamp: Date.now(),
      admin_context: true
    });

  } catch (error) {
    console.error('❌ Admin: Web3 challenge API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}