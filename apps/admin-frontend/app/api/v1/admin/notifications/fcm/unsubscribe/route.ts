import { NextRequest, NextResponse } from 'next/server';
import { getAuthUser } from '@/lib/server/auth';

interface AdminFCMUnsubscribeRequest {
  fcmToken: string;
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

    // Parse request body
    const body: AdminFCMUnsubscribeRequest = await request.json();
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
    const response = await fetch(`${backendUrl}/api/v1/fcm/tokens/unsubscribe`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${request.headers.get('Authorization')?.replace('Bearer ', '') || ''}`
      },
      body: JSON.stringify({
        fcm_token: fcmToken,
        user_id: adminUser.sub,
        admin_context: true
      })
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      console.error('Backend admin FCM unsubscription failed:', errorData);
      
      // Don't fail the request if backend unsubscription fails
      // The client-side token cleanup is more important
      console.warn('Backend admin unsubscription failed, but proceeding with client cleanup');
    }

    const data = response.ok ? await response.json() : null;
    
    return NextResponse.json({
      success: true,
      message: 'Successfully unsubscribed from admin notifications',
      data: {
        fcmToken: fcmToken,
        unsubscribedAt: new Date().toISOString(),
        backendSuccess: response.ok,
        adminContext: true
      }
    });

  } catch (error) {
    console.error('Admin FCM unsubscribe API error:', error);
    
    // Don't fail unsubscription due to server errors
    // Client-side cleanup is more important
    return NextResponse.json({
      success: true,
      message: 'Admin client unsubscribed successfully',
      data: {
        fcmToken: null,
        unsubscribedAt: new Date().toISOString(),
        backendSuccess: false,
        warning: 'Server communication failed, but admin client unsubscribed successfully'
      }
    });
  }
}