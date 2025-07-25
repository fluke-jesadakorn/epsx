import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { adminLogger } from '../../../../lib/logger';

const BACKEND_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    
    // Forward cookies to backend
    const cookieHeader = request.headers.get('cookie') || '';

    try {
      const response = await fetch(`${BACKEND_URL}/api/v1/authentication/me`, {
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
      
      // Transform backend response to expected admin frontend format
      const transformedUser = {
        user: {
          uid: userData.user_id,
          email: userData.email,
          roles: [userData.role],
          isAdmin: userData.role === 'admin' || userData.role === 'super_admin',
          customClaims: {
            role: userData.role.toUpperCase(),
          },
        },
      };
      
      return NextResponse.json(transformedUser);
    } catch (fetchError) {
      adminLogger.error('Backend not available for admin profile check', { error: fetchError instanceof Error ? fetchError.message : String(fetchError) });
      return NextResponse.json({ error: 'Backend connection failed' }, { status: 503 });
    }
  } catch (error) {
    adminLogger.error('Admin auth check error', { error: error instanceof Error ? error.message : String(error) });
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}