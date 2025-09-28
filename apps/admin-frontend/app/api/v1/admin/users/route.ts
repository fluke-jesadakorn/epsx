import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET() {
  try {
    // Get Bearer token from admin session cookies
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    const adminSession = cookieStore.get('admin_session')?.value;

    if (!accessToken || !adminSession) {
      return NextResponse.json(
        { error: 'Admin authentication required' },
        { status: 401 }
      );
    }
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${accessToken}`,
      'X-Admin-Context': 'true',
    };

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Backend users API failed: ${response.status} ${errorText}`);
      return NextResponse.json(
        { error: `Backend error: ${errorText}` }, 
        { status: response.status }
      );
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('Users API error:', error);
    return NextResponse.json(
      { 
        users: [],
        error: 'Failed to fetch users',
        message: 'User service currently unavailable' 
      }, 
      { status: 500 }
    );
  }
}