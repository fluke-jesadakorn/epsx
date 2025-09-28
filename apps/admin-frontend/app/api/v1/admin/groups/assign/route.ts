/**
 * Admin Group Assignment API Route
 * Assigns wallet users to permission groups through backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address, group_id, role } = body;

    if (!wallet_address || !group_id) {
      return NextResponse.json(
        { error: 'Wallet address and group ID are required' },
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

    console.log('🔄 Admin: Assigning wallet:', wallet_address.slice(0, 8) + '...', 'to group:', group_id);

    // Forward to backend group assignment endpoint
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/groups/assign`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        wallet_address,
        group_id,
        role: role || 'member',
        assigned_by: 'admin_interface'
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Group assignment failed' }));
      console.error('❌ Admin: Backend group assignment failed:', errorData);
      return NextResponse.json(errorData, { status: response.status });
    }

    const responseData = await response.json();
    
    console.log('✅ Admin: Group assignment successful');
    
    return NextResponse.json({
      success: true,
      message: `Wallet assigned to group "${group_id}"`,
      data: responseData
    });

  } catch (error) {
    console.error('❌ Admin: Group assignment error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}