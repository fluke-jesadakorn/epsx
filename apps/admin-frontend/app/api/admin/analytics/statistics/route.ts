import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    // Get search params from the original request
    const { searchParams } = new URL(request.url);
    const queryString = searchParams.toString();
    
    // Forward the request to the Rust backend with cookies
    const backendUrl = `${BACKEND_URL}/admin/stats${queryString ? `?${queryString}` : ''}`;
    const cookieHeader = request.headers.get('cookie') || '';
    
    console.log('Admin stats proxy: forwarding to', backendUrl);
    console.log('Admin stats proxy: cookies', cookieHeader ? 'present' : 'none');
    
    try {
      const response = await fetch(backendUrl, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Cookie': cookieHeader,
        },
      });

      console.log('Admin stats proxy: backend response status', response.status);

      if (!response.ok) {
        const error = await response.text();
        console.error('Admin stats proxy: backend error', error);
        
        // In development, return mock data if backend returns 401 (not authenticated)
        if (process.env.NODE_ENV === 'development' && response.status === 401) {
          console.log('Admin stats proxy: backend auth failed, returning mock data for development');
          const mockStats = {
            totalUsers: 150,
            verifiedUsers: 120,
            disabledUsers: 5,
            adminUsers: 3,
            verificationRate: 80.0
          };
          return NextResponse.json(mockStats);
        }
        
        return NextResponse.json(
          { error: error || 'Failed to fetch user statistics' },
          { status: response.status }
        );
      }

      const data = await response.json();
      console.log('Admin stats proxy: backend data received', Object.keys(data));
      return NextResponse.json(data);
    } catch (fetchError) {
      console.error('Admin stats proxy: backend not available:', fetchError);
      
      // For development, return mock data if backend is not available
      if (process.env.NODE_ENV === 'development') {
        console.log('Admin stats proxy: returning mock data for development');
        const mockStats = {
          totalUsers: 150,
          verifiedUsers: 120,
          disabledUsers: 5,
          adminUsers: 3,
          verificationRate: 80.0
        };
        return NextResponse.json(mockStats);
      }
      
      throw fetchError;
    }
  } catch (error) {
    console.error('Admin stats proxy error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}