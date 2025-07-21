import { NextRequest, NextResponse } from 'next/server';

/**
 * Legacy admin API route that redirects to the new admin frontend API
 * This ensures seamless migration of existing API calls
 */
export async function GET(request: NextRequest) {
  const adminUrl = process.env.NEXT_PUBLIC_ADMIN_FRONTEND_URL || 'http://localhost:3001';
  const { searchParams } = new URL(request.url);
  
  // Forward the request to admin frontend
  try {
    const response = await fetch(`${adminUrl}/api/admin/users?${searchParams.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        // Forward auth headers
        'Cookie': request.headers.get('cookie') || '',
      },
    });
    
    const data = await response.json();
    return NextResponse.json(data, { status: response.status });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to connect to admin service' },
      { status: 503 }
    );
  }
}

export async function POST(request: NextRequest) {
  const adminUrl = process.env.NEXT_PUBLIC_ADMIN_FRONTEND_URL || 'http://localhost:3001';
  const body = await request.text();
  
  // Forward the request to admin frontend
  try {
    const response = await fetch(`${adminUrl}/api/admin/users`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        // Forward auth headers
        'Cookie': request.headers.get('cookie') || '',
      },
      body,
    });
    
    const data = await response.json();
    return NextResponse.json(data, { status: response.status });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to connect to admin service' },
      { status: 503 }
    );
  }
}
