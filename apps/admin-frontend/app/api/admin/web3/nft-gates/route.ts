import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

// GET /api/admin/web3/nft-gates - Get all NFT gate configurations
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

    console.log('🎨 Admin: Fetching NFT gates');

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/nft-gates`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to fetch NFT gates' }));
      console.error('❌ Admin: Backend NFT gates fetch failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Retrieved NFT gates:', {
      count: data.nft_gates?.length || 0,
      total: data.total_count
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 NFT gates GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/admin/web3/nft-gates - Create new NFT gate
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { contract_address, contract_name, permissions, min_token_count, network } = body;

    // Validate required fields
    if (!contract_address || !contract_name || !permissions || !Array.isArray(permissions) || permissions.length === 0) {
      return NextResponse.json(
        { error: 'contract_address, contract_name, and permissions array are required' },
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

    console.log('🎨 Admin: Creating NFT gate:', {
      contract: contract_address,
      name: contract_name,
      permissions: permissions.length,
      network: network || 'ethereum',
      admin: sessionWalletAddress
    });

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/nft-gates`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        contract_address,
        contract_name,
        permissions,
        min_token_count: min_token_count || 1,
        network: network || 'ethereum'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to create NFT gate' }));
      console.error('❌ Admin: NFT gate creation failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Successfully created NFT gate:', {
      contract: contract_address,
      created: data.created_permissions?.length || 0
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 NFT gate creation error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}