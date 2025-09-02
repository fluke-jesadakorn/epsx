import { NextRequest, NextResponse } from 'next/server';
import { getAuthUser } from '@/lib/server/auth';

interface AdminFCMTestRequest {
  title: string;
  body: string;
  icon?: string;
  url?: string;
  adminType?: 'user_management' | 'system_alert' | 'security_warning' | 'analytics_report' | 'permission_change';
  priority?: 'low' | 'normal' | 'high' | 'critical';
  data?: Record<string, any>;
  fcmToken?: string;
  isAdminTest?: boolean;
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

    // Verify admin permissions for test notifications
    if (!adminUser.permissions?.includes('admin:notifications:test') && 
        !adminUser.permissions?.includes('admin:*:*')) {
      return NextResponse.json(
        { error: 'Insufficient permissions to send admin test notifications' },
        { status: 403 }
      );
    }

    // Parse request body
    const body: AdminFCMTestRequest = await request.json();
    const { title, body: messageBody, icon, url, adminType, priority, data, fcmToken } = body;

    if (!title || !messageBody) {
      return NextResponse.json(
        { error: 'Title and body are required' },
        { status: 400 }
      );
    }

    // Create admin test notification payload
    const adminTestNotification = {
      title: `[ADMIN TEST] ${title}`,
      body: messageBody,
      icon: icon || '/logo.png',
      url: url || '/notifications',
      data: {
        type: 'admin_test',
        adminType: adminType || 'system_alert',
        priority: priority || 'normal',
        notificationId: `admin-test-${Date.now()}`,
        timestamp: new Date().toISOString(),
        userId: adminUser.sub,
        adminContext: true,
        testMode: true,
        ...data
      }
    };

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend with admin context
    const response = await fetch(`${backendUrl}/api/v1/fcm/notifications/test`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '') || ''}`
      },
      body: JSON.stringify({
        user_id: adminUser.sub,
        fcm_token: fcmToken,
        notification: adminTestNotification,
        test_mode: true,
        admin_test: true,
        admin_type: adminType || 'system_alert',
        priority: priority || 'normal'
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend admin test notification failed:', errorData);
      
      return NextResponse.json(
        { error: errorData.error || 'Failed to send admin test notification' },
        { status: response.status }
      );
    }

    const responseData = await response.json();
    
    return NextResponse.json({
      success: true,
      message: 'Admin test notification sent successfully',
      data: {
        notificationId: adminTestNotification.data.notificationId,
        timestamp: adminTestNotification.data.timestamp,
        adminType: adminType,
        priority: priority,
        backendResponse: responseData
      }
    });

  } catch (error) {
    console.error('Admin FCM test notification API error:', error);
    
    return NextResponse.json(
      { 
        error: 'Internal server error',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}