import { NextRequest, NextResponse } from 'next/server';
import { getServerAuth } from '@/lib/auth-server';

/**
 * GET /api/v1/authentication/me - Get current authenticated user
 * Returns the current user's profile information
 */
export async function GET(request: NextRequest) {
  try {
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return NextResponse.json(
        { 
          error: 'Not authenticated',
          code: 'UNAUTHORIZED'
        },
        { status: 401 }
      );
    }

    const { user } = authResult;
    
    return NextResponse.json({
      user_id: user.user_id,
      email: user.email,
      role: user.role,
      permissions: user.permissions,
      subscription_tier: user.subscription_tier,
      package_tier: user.package_tier,
      expires_at: user.expires_at,
      session_type: user.session_type,
      display_name: user.displayName,
      photo_url: user.photoURL,
      email_verified: user.emailVerified
    });
  } catch (error) {
    console.error('Error getting current user:', error);
    
    // If there's an error (like backend not available), treat as not authenticated
    // This prevents 500 errors when the backend is down
    return NextResponse.json(
      { 
        error: 'Not authenticated',
        code: 'UNAUTHORIZED'
      },
      { status: 401 }
    );
  }
}