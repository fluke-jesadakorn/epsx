import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken } from '@/lib/actions/admin';
import { env } from '@/config/env';

const BACKEND_URL = env.NEXT_PUBLIC_BACKEND_URL || env.getBackendUrl();

export async function GET(request: NextRequest) {
  try {
    console.log('🔍 [Admin Modules API] Fetching user admin modules');

    // Get bearer token from request
    const bearerToken = await getBearerToken(request);
    
    if (!bearerToken) {
      console.log('❌ [Admin Modules API] No bearer token available');
      return NextResponse.json(
        { error: 'Authentication required' },
        { status: 401 }
      );
    }

    console.log('🔧 [Admin Modules API] Using bearer token for backend request');

    // Make request to backend
    const backendResponse = await fetch(`${BACKEND_URL}/api/v1/admin/modules/user`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${bearerToken}`,
        'Content-Type': 'application/json',
      },
    });

    console.log('📡 [Admin Modules API] Backend response status:', backendResponse.status);

    if (!backendResponse.ok) {
      const errorText = await backendResponse.text();
      console.log('❌ [Admin Modules API] Backend error response:', errorText);
      
      if (backendResponse.status === 401) {
        return NextResponse.json(
          { error: 'Authentication required' },
          { status: 401 }
        );
      }
      
      return NextResponse.json(
        { error: 'Failed to fetch admin modules' },
        { status: backendResponse.status }
      );
    }

    const data = await backendResponse.json();
    console.log('✅ [Admin Modules API] Successfully fetched admin modules:', data);

    return NextResponse.json(data);
    
  } catch (error) {
    console.error('🚨 [Admin Modules API] Error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}