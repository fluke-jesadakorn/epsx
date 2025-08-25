import { NextRequest, NextResponse } from 'next/server';
import { createApiClient } from '@/lib/api-client';
import { getAuthUser } from '@/lib/server/auth';

interface FCMUnsubscribeRequest {
  fcmToken: string;
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
    const body: FCMUnsubscribeRequest = await request.json();
    const { fcmToken } = body;

    if (!fcmToken) {
      return NextResponse.json(
        { error: 'FCM token is required' },
        { status: 400 }
      );
    }

    // Create API client with backend URL
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';

    // Forward request to backend
    const response = await fetch(`${backendUrl}/api/v1/notifications/fcm/unsubscribe`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '')}`
      },
      body: JSON.stringify({
        fcm_token: fcmToken,
        user_id: user.sub
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend FCM unsubscription failed:', errorData);
      
      // Don't fail the request if backend unsubscription fails
      // The client-side token cleanup is more important
      console.warn('Backend unsubscription failed, but proceeding with client cleanup');
    }

    const data = response.ok ? await response.json() : null;
    
    return NextResponse.json({
      success: true,
      data: {
        fcmToken: fcmToken,
        unsubscribedAt: new Date().toISOString(),
        backendSuccess: response.ok
      }
    });

  } catch (error) {
    console.error('FCM unsubscribe API error:', error);
    
    // Don't fail unsubscription due to server errors
    // Client-side cleanup is more important
    return NextResponse.json({
      success: true,
      data: {
        fcmToken: null,
        unsubscribedAt: new Date().toISOString(),
        backendSuccess: false,
        warning: 'Server communication failed, but client unsubscribed successfully'
      }
    });
  }
}