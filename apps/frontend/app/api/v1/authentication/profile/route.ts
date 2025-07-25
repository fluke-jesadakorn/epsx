import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { COOKIE_NAMES } from '@/lib/cookies';
import { logger } from '@/lib/logger';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    
    // Get session ID from our __session cookie and send it as sess_id to backend
    const sessionId = cookieStore.get(COOKIE_NAMES.SESSION)?.value;
    
    if (!sessionId) {
      return NextResponse.json({ error: 'Not authenticated' }, { status: 401 });
    }

    try {
      const response = await fetch(`${BACKEND_URL}/api/v1/authentication/profile`, {
        method: 'GET',
        headers: {
          'Cookie': `sess_id=${sessionId}`,
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
      logger.error('Backend not available for authentication', { error: fetchError instanceof Error ? fetchError.message : String(fetchError) });
      
      // TODO: Temporary development fallback when backend is not available
      if (process.env.NODE_ENV === 'development') {
        const sessionToken = cookieStore.get(COOKIE_NAMES.SESSION)?.value;
        
        if (sessionToken) {
          logger.info('Backend unavailable, returning mock user data for development');
          // Return mock user data for development
          const mockUserData = {
            user_id: 'dev-user-123',
            email: 'jesadakorn.kirtnu@gmail.com',
            role: 'user',
            permissions: ['analytics.view', 'data.own.view', 'settings.view'],
            subscription_tier: 'bronze',
            package_tier: 'BRONZE',
            expires_at: new Date(Date.now() + 86400000).toISOString(), // 24 hours from now
            session_type: 'dev_session'
          };
          
          return NextResponse.json(mockUserData);
        }
      }
      
      return NextResponse.json({ error: 'Not authenticated' }, { status: 401 });
    }
  } catch (error) {
    logger.error('Auth check error', { error: error instanceof Error ? error.message : String(error) });
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}