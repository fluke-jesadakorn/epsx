import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { serverPermissionCache } from '@/lib/server/permission-cache';

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

    // Verify wallet session exists (allow for initial login check)
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    // During login, wallet session might not exist yet - that's OK
    const isInitialLogin = !sessionWalletAddress;
    
    if (sessionWalletAddress && sessionWalletAddress.toLowerCase() !== wallet_address.toLowerCase()) {
      return NextResponse.json(
        { error: 'Wallet address mismatch' },
        { status: 401 }
      );
    }

    console.log('🔄 Admin: Fetching permissions for wallet:', wallet_address);

    // Use server-side cache to prevent heavy database queries
    const permissionData = await serverPermissionCache.getPermissions(wallet_address);
    
    console.log('✅ Admin: Permissions fetched for wallet:', wallet_address, 'Level:', permissionData.admin_level, 'All permissions:', permissionData.permissions);
    
    return NextResponse.json({
      wallet_address: permissionData.wallet_address,
      permissions: permissionData.permissions,
      admin_permissions: permissionData.permissions.filter(p => 
        p === 'admin:*:*' || 
        p.startsWith('admin:') ||
        p === 'epsx:admin:*' ||
        p === 'epsx:*:*'
      ),
      admin_level: permissionData.admin_level,
      has_admin_access: permissionData.has_admin_access,
      timestamp: permissionData.timestamp
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