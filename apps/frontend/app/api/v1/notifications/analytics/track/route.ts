import { NextRequest, NextResponse } from 'next/server';

interface NotificationTrackingData {
  notificationId: string;
  action: string;
  url?: string;
  timestamp: string;
  userAgent?: string;
}

export async function POST(request: NextRequest) {
  try {
    // Parse request body
    const trackingData: NotificationTrackingData = await request.json();
    const { notificationId, action, url, timestamp, userAgent } = trackingData;

    if (!notificationId || !action) {
      return NextResponse.json(
        { error: 'Notification ID and action are required' },
        { status: 400 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend
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
          source: 'fcm_service_worker',
          browser: 'unknown',
          platform: 'web'
        }
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend notification tracking failed:', errorData);
      
      // Don't fail the request for tracking failures
      return NextResponse.json({
        success: false,
        warning: 'Tracking data could not be stored',
        error: errorData.error
      });
    }

    const data = await response.json();
    
    return NextResponse.json({
      success: true,
      message: 'Tracking data stored successfully',
      data
    });

  } catch (error) {
    console.error('Notification tracking API error:', error);
    
    // Don't fail for tracking errors
    return NextResponse.json({
      success: false,
      warning: 'Tracking failed due to server error',
      error: error instanceof Error ? error.message : 'Unknown error'
    });
  }
}