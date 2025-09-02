import { NextRequest, NextResponse } from 'next/server';
import { getAuthUser } from '@/lib/server/auth';

export async function GET(request: NextRequest) {
  try {
    // Get authenticated admin user
    const adminUser = await getAuthUser();
    if (!adminUser) {
      return NextResponse.json(
        { error: 'Unauthorized - Admin access required' },
        { status: 401 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend
    const response = await fetch(`${backendUrl}/api/v1/fcm/tokens/my`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '') || ''}`
      }
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend get admin user tokens failed:', errorData);
      
      return NextResponse.json(
        { error: errorData.error || 'Failed to get admin user tokens' },
        { status: response.status }
      );
    }

    const data = await response.json();
    
    // Transform backend response to admin frontend format
    const tokens = data.tokens?.map((token: any) => ({
      token: token.fcm_token,
      createdAt: new Date(token.created_at),
      userAgent: token.device_info?.user_agent || 'Unknown',
      platform: token.device_info?.platform || 'Unknown',
      isActive: token.is_active,
      adminPermissions: token.device_info?.admin_permissions || []
    })) || [];
    
    return NextResponse.json({
      success: true,
      tokens,
      count: tokens.length,
      adminContext: true
    });

  } catch (error) {
    console.error('Admin FCM get user tokens API error:', error);
    
    return NextResponse.json(
      { 
        error: 'Internal server error',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}