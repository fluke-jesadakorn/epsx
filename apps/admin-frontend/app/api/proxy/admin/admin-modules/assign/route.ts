import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';
import { env } from '@/config/env';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.NEXT_PUBLIC_BACKEND_URL;

export async function POST(request: NextRequest) {
  try {
    const token = await getBearerToken();
    const body = await request.json();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/admin-modules/assign`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('Backend assignment error:', {
        status: response.status,
        statusText: response.statusText,
        error: errorText
      });
      
      return NextResponse.json(
        { error: `Assignment failed: ${response.statusText}` },
        { status: response.status }
      );
    }

    const data = await response.json();
    return NextResponse.json(data);
    
  } catch (error) {
    console.error('Admin module assignment API error:', error);
    return NextResponse.json(
      { error: 'Failed to assign admin modules' },
      { status: 500 }
    );
  }
}