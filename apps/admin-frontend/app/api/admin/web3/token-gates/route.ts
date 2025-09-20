import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

// GET /api/admin/web3/token-gates - Get all token gate configurations
export async function GET(request: NextRequest) {
  try {
    // Verify admin session
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🪙 Admin: Fetching token gates');

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/token-gates`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to fetch token gates' }));
      console.error('❌ Admin: Backend token gates fetch failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Retrieved token gates:', {
      count: data.token_gates?.length || 0,
      total: data.total_count
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 token gates GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/admin/web3/token-gates - Create new token gate
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { contract_address, token_symbol, permissions, min_amount, token_decimals, network } = body;

    // Validate required fields
    if (!contract_address || !token_symbol || !permissions || !Array.isArray(permissions) || permissions.length === 0 || !min_amount) {
      return NextResponse.json(
        { error: 'contract_address, token_symbol, permissions array, and min_amount are required' },
        { status: 400 }
      );
    }

    // Verify admin session
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🪙 Admin: Creating token gate:', {
      contract: contract_address,
      symbol: token_symbol,
      permissions: permissions.length,
      minAmount: min_amount,
      network: network || 'ethereum',
      admin: sessionWalletAddress
    });

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/token-gates`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        contract_address,
        token_symbol,
        permissions,
        min_amount,
        token_decimals: token_decimals || 18,
        network: network || 'ethereum'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to create token gate' }));
      console.error('❌ Admin: Token gate creation failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Successfully created token gate:', {
      contract: contract_address,
      symbol: token_symbol,
      created: data.created_permissions?.length || 0
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 token gate creation error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}