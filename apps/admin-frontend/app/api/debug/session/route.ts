import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';

export async function GET(request: NextRequest) {
  try {
    console.log('🔍 Debug session endpoint called');
    
    const session = await getServerSession();
    
    console.log('🔍 NextAuth session data:', {
      hasSession: !!session,
      sessionKeys: session ? Object.keys(session) : 'no session',
      userId: session?.user?.id || 'no user_id',
      userEmail: session?.user?.email || 'no user email'
    });

    return NextResponse.json({
      success: true,
      session: session ? {
        hasSession: true,
        user: session.user,
        permissions: (session.user as any)?.permissions || [],
        expires: session.expires,
        keys: Object.keys(session)
      } : { hasSession: false }
    });
  } catch (error) {
    console.error('🔍 Debug session error:', error);
    return NextResponse.json({
      success: false,
      error: error instanceof Error ? error.message : String(error)
    });
  }
}