import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

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

    // Verify wallet session exists
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress || sessionWalletAddress.toLowerCase() !== wallet_address.toLowerCase()) {
      return NextResponse.json(
        { error: 'Invalid or missing wallet session' },
        { status: 401 }
      );
    }

    console.log('🔄 Admin: Fetching permissions for wallet:', wallet_address);

    // Forward to backend Web3 permissions endpoint
    const response = await fetch(`${BACKEND_URL}/api/auth/web3/permissions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Admin-Context': 'true',
        'Authorization': `Bearer ${sessionWalletAddress}`, // Use wallet address as token
      },
      body: JSON.stringify({ 
        wallet_address,
        admin_context: true
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to fetch permissions' }));
      console.error('❌ Admin: Backend permissions fetch failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    
    // Filter and validate admin permissions
    const allPermissions = data.permissions || [];
    const adminPermissions = allPermissions.filter((permission: string) => 
      permission === 'admin:*:*' || 
      permission.startsWith('admin:') ||
      permission === 'epsx:admin:*'
    );
    
    // Determine admin level based on permissions
    let adminLevel = 'none';
    if (adminPermissions.includes('admin:*:*')) {
      adminLevel = 'super';
    } else if (adminPermissions.some((p: string) => p.includes('admin:web3:manage'))) {
      adminLevel = 'manager';
    } else if (adminPermissions.length > 0) {
      adminLevel = 'moderator';
    }
    
    console.log('✅ Admin: Permissions fetched for wallet:', wallet_address, 'Level:', adminLevel);
    
    return NextResponse.json({
      wallet_address,
      permissions: allPermissions,
      admin_permissions: adminPermissions,
      admin_level: adminLevel,
      has_admin_access: adminPermissions.length > 0,
      timestamp: Date.now()
    });

  } catch (error) {
    console.error('❌ Admin: Web3 permissions API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  try {
    // Get wallet from session cookie
    const cookieStore = await cookies();
    const walletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!walletAddress) {
      return NextResponse.json(
        { error: 'No wallet session found' },
        { status: 401 }
      );
    }

    // Re-use POST logic with wallet from session
    const mockRequest = {
      json: () => Promise.resolve({ wallet_address: walletAddress })
    } as NextRequest;
    
    return await POST(mockRequest);
    
  } catch (error) {
    console.error('❌ Admin: Web3 permissions GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}