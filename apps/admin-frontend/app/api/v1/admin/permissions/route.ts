/**
 * Admin Permissions Management API Route
 * Manages wallet permissions with filtering and pagination
 * Proxies to backend /api/v1/auth/web3/permissions
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET(request: NextRequest) {
  try {
    // Get search params for filtering
    const searchParams = request.nextUrl.searchParams;
    const walletAddress = searchParams.get('wallet_address');
    const permission = searchParams.get('permission');
    const source = searchParams.get('source');
    const limitParam = searchParams.get('limit');
    const offsetParam = searchParams.get('offset');

    // Verify admin session
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const adminSession = cookieStore.get('admin_session')?.value;
    
    if (!accessToken || !adminSession) {
      return NextResponse.json(
        { error: 'Admin authentication required' },
        { status: 401 }
      );
    }

    console.log('🔍 Admin: Fetching Web3 permissions with filters:', {
      walletAddress: walletAddress?.slice(0, 8) + '...',
      permission,
      source,
      limit: limitParam,
      offset: offsetParam
    });

    // Build query parameters for backend API
    const params = new URLSearchParams();
    if (walletAddress) params.append('wallet_address', walletAddress);
    if (permission) params.append('permission', permission);
    if (source) params.append('source', source);
    if (limitParam) params.append('limit', limitParam);
    if (offsetParam) params.append('offset', offsetParam);

    // Forward to backend permissions endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/permissions?${params.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
    });

    if (!response.ok) {
      console.error(`❌ Admin: Backend permissions API failed: ${response.status}`);
      return NextResponse.json(
        { error: 'Failed to fetch permissions from backend' },
        { status: response.status }
      );
    }

    const permissionsData = await response.json();
    
    console.log('✅ Admin: Retrieved permissions from backend:', {
      count: permissionsData.permissions?.length || 0,
      total: permissionsData.total_count || 0
    });

    return NextResponse.json(permissionsData);

  } catch (error) {
    console.error('❌ Admin: Permissions API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    // Verify admin session
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const adminSession = cookieStore.get('admin_session')?.value;
    
    if (!accessToken || !adminSession) {
      return NextResponse.json(
        { error: 'Admin authentication required' },
        { status: 401 }
      );
    }

    const body = await request.json();

    // Forward permission grant request to backend
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/permissions/grant`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Permission grant failed' 
      }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const result = await response.json();
    return NextResponse.json(result);

  } catch (error) {
    console.error('❌ Admin: Permission grant error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: NextRequest) {
  try {
    // Verify admin session
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const adminSession = cookieStore.get('admin_session')?.value;
    
    if (!accessToken || !adminSession) {
      return NextResponse.json(
        { error: 'Admin authentication required' },
        { status: 401 }
      );
    }

    const body = await request.json();

    // Forward permission revoke request to backend
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/web3/permissions/revoke`, {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: 'Permission revoke failed' 
      }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const result = await response.json();
    return NextResponse.json(result);

  } catch (error) {
    console.error('❌ Admin: Permission revoke error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}