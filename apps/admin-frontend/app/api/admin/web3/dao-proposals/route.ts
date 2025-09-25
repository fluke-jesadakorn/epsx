import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

// GET /api/admin/web3/dao-proposals - Get all DAO proposals
export async function GET(request: NextRequest) {
  try {
    // Verify admin session - simple wallet_address check
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🗳️ Admin: Fetching DAO proposals for:', sessionWalletAddress);

    // Return empty DAO proposals for now - this feature can be implemented later
    // This allows the component to load without errors
    const data = {
      proposals: [],
      total_count: 0
    };

    console.log('✅ Admin: Retrieved DAO proposals:', {
      count: data.proposals.length,
      total: data.total_count
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 DAO proposals GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/admin/web3/dao-proposals - Create new DAO proposal
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { title, description, target_wallet, permissions, votes_required, expires_at, dao_contract_address, network } = body;

    // Validate required fields
    if (!title || !target_wallet || !permissions || !Array.isArray(permissions) || permissions.length === 0 || !expires_at) {
      return NextResponse.json(
        { error: 'title, target_wallet, permissions array, and expires_at are required' },
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

    console.log('🗳️ Admin: Creating DAO proposal:', {
      title,
      target: target_wallet,
      permissions: permissions.length,
      votesRequired: votes_required || 3,
      network: network || 'ethereum',
      admin: sessionWalletAddress
    });

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${env.BACKEND_URL}/admin/web3/dao-proposals`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        title,
        description: description || '',
        target_wallet,
        permissions,
        votes_required: votes_required || 3,
        expires_at,
        dao_contract_address,
        network: network || 'ethereum'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to create DAO proposal' }));
      console.error('❌ Admin: DAO proposal creation failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Successfully created DAO proposal:', {
      title,
      target: target_wallet,
      created: data.created_permissions?.length || 0
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 DAO proposal creation error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}