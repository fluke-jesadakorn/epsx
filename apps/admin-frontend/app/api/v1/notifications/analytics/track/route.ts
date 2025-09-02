import { NextRequest, NextResponse } from 'next/server';

interface AdminNotificationTrackingData {
  notificationId: string;
  action: string;
  url?: string;
  timestamp: string;
  userAgent?: string;
  context?: string;
  platform?: string;
}

export async function POST(request: NextRequest) {
  try {
    // Parse request body
    const trackingData: AdminNotificationTrackingData = await request.json();
    const { notificationId, action, url, timestamp, userAgent, context } = trackingData;

    if (!notificationId || !action) {
      return NextResponse.json(
        { error: 'Notification ID and action are required' },
        { status: 400 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend with admin context
    const response = await fetch(`${backendUrl}/api/v1/notifications/analytics/track`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        notification_id: notificationId,
        action,
        url,
        timestamp,
        user_agent: userAgent,
        tracking_metadata: {
          source: 'admin_fcm_service_worker',
          browser: 'unknown',
          platform: 'web',
          context: context || 'admin',
          admin_interface: true
        }
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend admin notification tracking failed:', errorData);
      
      // Don't fail the request for tracking failures
      return NextResponse.json({
        success: false,
        warning: 'Admin tracking data could not be stored',
        error: errorData.error
      });
    }

    const data = await response.json();
    
    return NextResponse.json({
      success: true,
      message: 'Admin tracking data stored successfully',
      data
    });

  } catch (error) {
    console.error('Admin notification tracking API error:', error);
    
    // Don't fail for tracking errors
    return NextResponse.json({
      success: false,
      warning: 'Admin tracking failed due to server error',
      error: error instanceof Error ? error.message : 'Unknown error'
    });
  }
}