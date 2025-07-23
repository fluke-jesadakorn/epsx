import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    
    // Forward cookies to backend
    const cookieHeader = request.headers.get('cookie') || '';

    try {
      const response = await fetch(`${BACKEND_URL}/auth/me`, {
        method: 'GET',
        headers: {
          'Cookie': cookieHeader,
        },
      });

      if (!response.ok) {
        return NextResponse.json(
          { error: 'Not authenticated' }, 
          { status: response.status }
        );
      }

      const userData = await response.json();
      return NextResponse.json(userData);
    } catch (fetchError) {
      console.error('Backend not available:', fetchError);
      
      // TODO: Temporary development fallback when backend is not available
      if (process.env.NODE_ENV === 'development') {
        const sessionToken = cookieStore.get('__session')?.value;
        
        if (sessionToken) {
          console.log('Backend unavailable, returning mock user data');
          // Return mock user data for development
          const mockUserData = {
            user_id: 'dev-user-123',
            email: 'jesadakorn.kirtnu@gmail.com',
            role: 'user',
            permissions: ['analytics.view', 'data.own.view', 'settings.view'],
            subscription_tier: 'bronze',
            package_tier: 'BRONZE',
            expires_at: new Date(Date.now() + 86400000).toISOString(), // 24 hours from now
            session_type: 'firebase_dev'
          };
          
          return NextResponse.json(mockUserData);
        }
      }
      
      return NextResponse.json({ error: 'Not authenticated' }, { status: 401 });
    }
  } catch (error) {
    console.error('Auth check error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}