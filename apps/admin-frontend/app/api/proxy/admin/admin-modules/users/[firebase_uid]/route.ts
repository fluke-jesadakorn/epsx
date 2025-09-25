import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';
import { env } from '@/config/env';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.BACKEND_URL;

interface RouteParams {
  params: {
    firebase_uid: string;
  };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const { firebase_uid } = await params;
    const token = await getBearerToken();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(
      `${BACKEND_URL}/api/v1/admin/admin-modules/users/${encodeURIComponent(firebase_uid)}`,
      {
        method: 'GET',
        headers,
      }
    );

    if (!response.ok) {
      if (response.status === 404) {
        // User not found or no assignments
        return NextResponse.json({
          firebase_uid,
          modules: [],
          module_details: [],
          is_admin: false,
          total_modules: 0
        });
      }
      throw new Error(`Backend request failed: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return NextResponse.json(data);
    
  } catch (error) {
    console.error('User admin modules API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch user admin modules' },
      { status: 500 }
    );
  }
}