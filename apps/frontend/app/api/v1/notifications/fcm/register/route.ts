import { NextRequest, NextResponse } from 'next/server';
import { getBackendUrl } from '../../../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { token, platform, device_info } = body;

    if (!token) {
      return NextResponse.json(
        { error: 'FCM token is required' },
        { status: 400 }
      );
    }

    // Forward request to Rust backend
    const backendResponse = await fetch(`${getBackendUrl('server')}/api/v1/notifications/fcm/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        // Forward auth cookies/headers
        'Cookie': request.headers.get('cookie') || '',
        'Authorization': request.headers.get('authorization') || '',
      },
      body: JSON.stringify({
        token,
        platform: platform || 'web',
        device_info: {
          userAgent: device_info?.userAgent,
          language: device_info?.language,
          timezone: device_info?.timezone,
          ...device_info
        }
      }),
    });

    if (!backendResponse.ok) {
      const errorData = await backendResponse.text();
      console.error('Backend registration failed:', errorData);
      return NextResponse.json(
        { error: 'Failed to register FCM token' },
        { status: backendResponse.status }
      );
    }

    const data = await backendResponse.json();
    
    return NextResponse.json({
      success: true,
      message: 'FCM token registered successfully',
      data
    });

  } catch (error) {
    console.error('FCM registration error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}