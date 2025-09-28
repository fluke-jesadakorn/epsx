/**
 * Admin Permission Grant API Route
 * Grants permissions to wallet users through backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, permission, expires_at, notes } = body;

    if (!wallet_address || !permission) {
      return NextResponse.json(
        { error: 'Wallet address and permission are required' },
        { status: 400 }
      );
    }

    // Verify admin session
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🔄 Admin: Granting permission:', permission, 'to wallet:', wallet_address.slice(0, 8) + '...');

    // Forward to backend permission grant endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/permissions/grant`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        wallet_address,
        permission,
        expires_at,
        notes,
        granted_by: 'admin_interface'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Permission grant failed' }));
      console.error('❌ Admin: Backend permission grant failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const responseData = await response.json();
    
    console.log('✅ Admin: Permission granted successfully');
    
    return NextResponse.json({
      success: true,
      message: `Permission "${permission}" granted to ${wallet_address}`,
      data: responseData
    });

  } catch (error) {
    console.error('❌ Admin: Permission grant error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}