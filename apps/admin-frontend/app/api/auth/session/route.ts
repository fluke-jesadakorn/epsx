/**
 * Session API Route
 * Returns current user session data
 */
import { NextRequest, NextResponse } from 'next/server';
import { getSessionFromRequest, refreshSessionIfNeeded, getSessionFromCookie } from '@/lib/auth/session';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    console.log('🔍 Session check requested');

    // Try manual session cookie first (new approach)
    const cookieStore = await cookies();
    const sessionCookieValue = cookieStore.get('epsx-admin-session')?.value;
    
    let session;
    if (sessionCookieValue) {
      console.log('🔍 Found manual session cookie, attempting to read...');
      session = getSessionFromCookie(sessionCookieValue);
    } else {
      console.log('🔍 No manual session cookie found, trying iron-session...');
      // Fallback to iron-session approach
      session = await getSessionFromRequest(request);
    }

    // Refresh session if needed
    session = await refreshSessionIfNeeded(session);

    // Return session data (excluding sensitive tokens for client-side use)
    const clientSession = {
      user: session.user || null,
      isLoggedIn: session.isLoggedIn,
      expiresAt: session.expiresAt,
    };

    console.log('🔍 Session check result:', {
      isLoggedIn: clientSession.isLoggedIn,
      userId: clientSession.user?.id,
      email: clientSession.user?.email,
      source: sessionCookieValue ? 'manual-cookie' : 'iron-session',
    });

    return NextResponse.json(clientSession);
  } catch (error) {
    console.error('❌ Session check error:', error);
    
    // Return empty session on error
    return NextResponse.json({
      user: null,
      isLoggedIn: false,
      expiresAt: null,
    });
  }
}