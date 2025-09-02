import { NextRequest, NextResponse } from 'next/server';
import { getAuthUser } from '@/lib/server/auth';

interface FCMTestRequest {
  title: string;
  body: string;
  icon?: string;
  url?: string;
  data?: Record<string, any>;
  fcmToken?: string;
}

export async function POST(request: NextRequest) {
  try {
    // Get authenticated user
    const user = await getAuthUser();
    if (!user) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }

    // Parse request body
    const body: FCMTestRequest = await request.json();
    const { title, body: messageBody, icon, url, data, fcmToken } = body;

    if (!title || !messageBody) {
      return NextResponse.json(
        { error: 'Title and body are required' },
        { status: 400 }
      );
    }

    // Create test notification payload
    const testNotification = {
      title,
      body: messageBody,
      icon: icon || '/logo.png',
      url: url || '/notifications',
      data: {
        type: 'test',
        notificationId: `test-${Date.now()}`,
        timestamp: new Date().toISOString(),
        userId: user.sub,
        ...data
      }
    };

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend
    const response = await fetch(`${backendUrl}/api/v1/fcm/notifications/test`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '') || ''}`
      },
      body: JSON.stringify({
        user_id: user.sub,
        fcm_token: fcmToken,
        notification: testNotification,
        test_mode: true
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend test notification failed:', errorData);
      
      return NextResponse.json(
        { error: errorData.error || 'Failed to send test notification' },
        { status: response.status }
      );
    }

    const responseData = await response.json();
    
    return NextResponse.json({
      success: true,
      message: 'Test notification sent successfully',
      data: {
        notificationId: testNotification.data.notificationId,
        timestamp: testNotification.data.timestamp,
        backendResponse: responseData
      }
    });

  } catch (error) {
    console.error('FCM test notification API error:', error);
    
    return NextResponse.json(
      { 
        error: 'Internal server error',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}