/**
 * Frontend Session API Route
 * Returns current user session data
 */
import { NextRequest, NextResponse } from 'next/server';
import { getSession } from '@/lib/auth/session';

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Fetching current session');

    // Get current session
    const session = await getSession();

    console.log('✅ Frontend: Session retrieved:', {
      isLoggedIn: session.isLoggedIn,
      userEmail: session.user?.email,
    });

    return NextResponse.json(session);

  } catch (error) {
    console.error('❌ Frontend: Session retrieval failed:', error);
    
    return NextResponse.json(
      { 
        isLoggedIn: false,
        user: null,
        accessToken: null,
      },
      { status: 200 }
    );
  }
}