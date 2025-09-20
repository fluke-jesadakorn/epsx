import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

// GET /api/admin/web3/permissions - Get wallet permissions with filtering
export async function GET(request: NextRequest) {
  try {
    // Get search params for filtering
    const searchParams = request.nextUrl.searchParams;
    const walletAddress = searchParams.get('wallet_address');
    const permission = searchParams.get('permission');
    const source = searchParams.get('source');
    const limit = searchParams.get('limit');
    const offset = searchParams.get('offset');

    // Verify admin session
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🔍 Admin: Fetching Web3 permissions with filters:', {
      walletAddress,
      permission,
      source,
      limit,
      offset
    });

    // Build query parameters for backend
    const backendParams = new URLSearchParams();
    if (walletAddress) backendParams.set('wallet_address', walletAddress);
    if (permission) backendParams.set('permission', permission);
    if (source) backendParams.set('source', source);
    if (limit) backendParams.set('limit', limit);
    if (offset) backendParams.set('offset', offset);

    // Forward to backend Web3 admin endpoint
    const response = await fetch(`${BACKEND_URL}/admin/web3/permissions?${backendParams.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionWalletAddress}`,
        'X-Admin-Context': 'true',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to fetch permissions' }));
      console.error('❌ Admin: Backend permissions fetch failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    console.log('✅ Admin: Retrieved permissions data:', {
      count: data.permissions?.length || 0,
      total: data.total_count
    });

    return NextResponse.json(data);

  } catch (error) {
    console.error('❌ Admin: Web3 permissions GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/admin/web3/permissions - Not used (permissions are granted via specific endpoints)
export async function POST(request: NextRequest) {
  return NextResponse.json(
    { error: 'Use specific permission grant endpoints instead' },
    { status: 405 }
  );
}