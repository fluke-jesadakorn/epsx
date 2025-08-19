import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';
import { env } from '@/config/env';

// Get bearer token from NextAuth session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.NEXT_PUBLIC_BACKEND_URL;

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
      // If backend route doesn't exist, return mock users
      if (response.status === 404) {
        return NextResponse.json({ 
          users: [
            {
              id: 'mock-user-1',
              email: 'user1@example.com',
              full_name: 'Mock User 1',
              status: 'active',
              created_at: new Date().toISOString(),
            },
            {
              id: 'mock-user-2', 
              email: 'user2@example.com',
              full_name: 'Mock User 2',
              status: 'active',
              created_at: new Date().toISOString(),
            }
          ],
          total: 2,
          message: 'Mock users (backend not implemented)'
        });
      }
      
      const errorText = await response.text();
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