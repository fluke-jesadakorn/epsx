import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';
import { env } from '@/config/env';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.BACKEND_URL;

export async function GET() {
  try {
    const token = await getBearerToken();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

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