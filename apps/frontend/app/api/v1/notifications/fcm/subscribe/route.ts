import { NextRequest, NextResponse } from 'next/server';
import { createApiClient } from '@/lib/api-client';
import { getAuthUser } from '@/lib/server/auth';

interface FCMSubscribeRequest {
  fcmToken: string;
  userAgent: string;
  platform: string;
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
    const body: FCMSubscribeRequest = await request.json();
    const { fcmToken, userAgent, platform } = body;

    if (!fcmToken) {
      return NextResponse.json(
        { error: 'FCM token is required' },
        { status: 400 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
    const apiClient = createApiClient(backendUrl);

    // Forward request to backend
    const response = await fetch(`${backendUrl}/api/v1/notifications/fcm/subscribe`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '')}`
      },
      body: JSON.stringify({
        fcm_token: fcmToken,
        user_id: user.sub,
        user_agent: userAgent,
        platform: platform,
        device_info: {
          user_agent: userAgent,
          platform: platform,
          subscribed_at: new Date().toISOString()
        }
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend FCM subscription failed:', errorData);
      
      return NextResponse.json(
        { error: errorData.error || 'Failed to subscribe to push notifications' },
        { status: response.status }
      );
    }

    const data = await response.json();
    
    return NextResponse.json({
      success: true,
      data: {
        subscriptionId: data.id,
        fcmToken: fcmToken,
        subscribedAt: new Date().toISOString()
      }
    });

  } catch (error) {
    console.error('FCM subscribe API error:', error);
    
    return NextResponse.json(
      { 
        error: 'Internal server error',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}