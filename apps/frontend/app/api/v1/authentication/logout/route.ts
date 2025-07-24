import { NextRequest, NextResponse } from 'next/server';
import { ApiCookies } from '@/lib/cookies';
import { destroySession } from '@/lib/session';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    // Forward cookies to backend for session cleanup
    const cookieHeader = request.headers.get('cookie') || '';

    // Try to logout from backend
    try {
      const response = await fetch(`${BACKEND_URL}/auth/logout`, {
        method: 'POST',
        headers: {
          'Cookie': cookieHeader,
        },
      });

      // Don't fail if backend logout fails
      if (!response.ok) {
        console.warn('Backend logout failed, continuing with client logout');
      }
    } catch (backendError) {
      console.warn('Backend not available for logout, continuing with client logout:', backendError);
    }

    // Create response
    const nextResponse = NextResponse.json({ success: true });

    // Clear all authentication cookies using our secure cookie utilities
    ApiCookies.clearAuthCookies(nextResponse);

    // Also destroy session using session utilities
    try {
      await destroySession();
    } catch (sessionError) {
      console.warn('Session destruction failed:', sessionError);
    }

    return nextResponse;
  } catch (error) {
    console.error('Logout error:', error);
    
    // Even if there's an error, try to clear cookies
    const nextResponse = NextResponse.json(
      { error: 'Logout failed, but cookies cleared' },
      { status: 500 }
    );
    
    ApiCookies.clearAuthCookies(nextResponse);
    
    return nextResponse;
  }
}
