import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Transform request to match backend format
    const backendBody = {
      type: "admin",
      email: body.email,
      password: body.password,
    };
    
    // Forward login request to backend
    const response = await fetch(`${BACKEND_URL}/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(backendBody),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: 'Login failed' }));
      return NextResponse.json(
        { message: errorData.message || 'Invalid credentials' },
        { status: response.status }
      );
    }

    const userData = await response.json();
    
    // Create response
    const nextResponse = NextResponse.json({
      user: {
        uid: userData.user_id,
        email: userData.email,
        roles: [userData.role],
        isAdmin: userData.role === 'admin' || userData.role === 'super_admin',
        customClaims: {
          role: userData.role.toUpperCase(),
        },
      },
    });
    
    // Forward session cookies from backend
    const setCookieHeader = response.headers.get('set-cookie');
    if (setCookieHeader) {
      nextResponse.headers.set('set-cookie', setCookieHeader);
    }

    return nextResponse;
  } catch (error) {
    console.error('Login error:', error);
    return NextResponse.json(
      { message: 'Internal server error' },
      { status: 500 }
    );
  }
}