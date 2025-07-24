import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    // Get search params from the original request
    const { searchParams } = new URL(request.url);
    const queryString = searchParams.toString();
    
    // Forward the request to the Rust backend with cookies
    const backendUrl = `${BACKEND_URL}/admin/users${queryString ? `?${queryString}` : ''}`;
    const cookieHeader = request.headers.get('cookie') || '';
    
    console.log('Admin users proxy: forwarding to', backendUrl);
    console.log('Admin users proxy: cookies', cookieHeader ? 'present' : 'none');
    
    const response = await fetch(backendUrl, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
    });

    console.log('Admin users proxy: backend response status', response.status);

    if (!response.ok) {
      const error = await response.text();
      console.error('Admin users proxy: backend error', error);
      
      return NextResponse.json(
        { error: error || 'Failed to fetch users' },
        { status: response.status }
      );
    }

    const data = await response.json();
    console.log('Admin users proxy: backend data received', `${data.users?.length || 0} users`);
    return NextResponse.json(data);
  } catch (error) {
    console.error('Admin users proxy error:', error);
    return NextResponse.json(
      { error: 'Backend connection failed' },
      { status: 503 }
    );
  }
}