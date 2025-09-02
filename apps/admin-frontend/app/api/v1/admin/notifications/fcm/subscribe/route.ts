import { NextRequest, NextResponse } from 'next/server';
import { getAuthUser } from '@/lib/server/auth';

interface AdminFCMSubscribeRequest {
  fcmToken: string;
  userAgent: string;
  platform: string;
  adminPermissions: string[];
  adminRole: string;
}

export async function POST(request: NextRequest) {
  try {
    // Get authenticated admin user
    const adminUser = await getAuthUser();
    if (!adminUser) {
      return NextResponse.json(
        { error: 'Unauthorized - Admin access required' },
        { status: 401 }
      );
    }

    // Verify admin permissions
    if (!adminUser.permissions?.includes('admin:notifications:manage') && 
        !adminUser.permissions?.includes('admin:*:*')) {
      return NextResponse.json(
        { error: 'Insufficient permissions for admin notifications' },
        { status: 403 }
      );
    }

    // Parse request body
    const body: AdminFCMSubscribeRequest = await request.json();
    const { fcmToken, userAgent, platform, adminPermissions, adminRole } = body;

    if (!fcmToken) {
      return NextResponse.json(
        { error: 'FCM token is required' },
        { status: 400 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend with admin context
    const response = await fetch(`${backendUrl}/api/v1/fcm/tokens/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '') || ''}`
      },
      body: JSON.stringify({
        fcm_token: fcmToken,
        user_id: adminUser.sub,
        user_agent: userAgent,
        platform: platform,
        device_info: {
          user_agent: userAgent,
          platform: platform,
          subscribed_at: new Date().toISOString(),
          is_admin: true,
          admin_role: adminRole,
          admin_permissions: adminPermissions
        },
        notification_types: [
          'admin_user_management',
          'admin_system_alert',
          'admin_security_warning',
          'admin_analytics_report',
          'admin_permission_change'
        ]
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend admin FCM subscription failed:', errorData);
      
      return NextResponse.json(
        { error: errorData.error || 'Failed to subscribe to admin push notifications' },
        { status: response.status }
      );
    }

    const data = await response.json();
    
    return NextResponse.json({
      success: true,
      message: 'Successfully subscribed to admin notifications',
      data: {
        subscriptionId: data.id,
        fcmToken: fcmToken,
        subscribedAt: new Date().toISOString(),
        adminContext: {
          role: adminRole,
          permissions: adminPermissions
        }
      }
    });

  } catch (error) {
    console.error('Admin FCM subscribe API error:', error);
    
    return NextResponse.json(
      { 
        error: 'Internal server error',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}