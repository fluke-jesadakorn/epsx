/**
 * Admin Frontend Session API Route
 * Handles JWT session management via secure cookies
 */
import { NextRequest, NextResponse } from 'next/server';
import { getSessionFromJWT } from '@/lib/server/jwt';
import { createCookieManager } from '@/lib/auth/cookie-manager';

export async function GET() {
  try {
    // Get session data from JWT cookie
    const sessionData = await getSessionFromJWT();
    
    if (!sessionData.isAuthenticated || !sessionData.user) {
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'No valid session found'
      }, { status: 401 });
    }

    // Return session data for client-side hooks
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        id: sessionData.user.sub,
        email: sessionData.user.email,
        name: sessionData.user.name,
        role: sessionData.user.role,
        admin_modules: sessionData.user.admin_modules,
        permissions: sessionData.user.permissions,
        package_tier: sessionData.user.package_tier,
        firebase_uid: sessionData.user.firebase_uid,
      },
      expiresAt: sessionData.user.exp * 1000, // Convert to milliseconds
    });

  } catch (error) {
    console.error('❌ Session API error:', error);
    
    return NextResponse.json({
      isAuthenticated: false,
      user: null,
      error: 'Session verification failed'
    }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { accessToken } = body;
    
    if (!accessToken) {
      return NextResponse.json({
        error: 'Access token required'
      }, { status: 400 });
    }
    
    const cookieManager = createCookieManager('admin');
    const response = NextResponse.json({ success: true });
    
    // Store JWT in secure cookie
    await cookieManager.setAccessToken(response, accessToken);
    
    return response;
    
  } catch (error) {
    console.error('❌ Session store error:', error);
    return NextResponse.json({
      error: 'Failed to store session'
    }, { status: 500 });
  }
}

export async function PUT() {
  try {
    // Get current session to check if refresh is needed
    const sessionData = await getSessionFromJWT();
    
    if (!sessionData.isAuthenticated || !sessionData.user) {
      return NextResponse.json({
        error: 'No valid session to refresh'
      }, { status: 401 });
    }
    
    // For now, we'll implement token refresh later
    // This is a placeholder for future refresh token functionality
    return NextResponse.json({
      message: 'Token refresh not yet implemented',
      currentExp: sessionData.user.exp
    });
    
  } catch (error) {
    console.error('❌ Session refresh error:', error);
    return NextResponse.json({
      error: 'Failed to refresh session'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const cookieManager = createCookieManager('admin');
    const response = NextResponse.json({ success: true });
    
    // Clear all auth cookies
    cookieManager.clearAllCookies(response);
    
    return response;
    
  } catch (error) {
    console.error('❌ Session clear error:', error);
    return NextResponse.json({
      error: 'Failed to clear session'
    }, { status: 500 });
  }
}