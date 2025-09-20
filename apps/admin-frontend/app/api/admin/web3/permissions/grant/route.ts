import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

// POST /api/admin/web3/permissions/grant - Grant manual permissions to wallet
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, permissions, expires_at, grant_reason } = body;

    // Validate required fields
    if (!wallet_address || !permissions || !Array.isArray(permissions) || permissions.length === 0) {
      return NextResponse.json(
        { error: 'wallet_address and permissions array are required' },
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

    console.log('🎯 Admin: Granting manual permissions:', {
      target: wallet_address,
      permissions: permissions.length,
      expires_at,
      admin: sessionWalletAddress
    });

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/permissions/grant`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        wallet_address,
        permissions,
        expires_at,
        grant_reason
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to grant permissions' }));
      console.error('❌ Admin: Permission grant failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Successfully granted permissions:', {
      wallet: wallet_address,
      granted: data.granted_permissions?.length || 0
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 permission grant error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}