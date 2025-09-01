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
        permissions: sessionData.user.permissions || [],
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
    
    const payload = sessionData.user;
    
    // Check if token is close to expiring (less than 5 minutes left)
    const now = Math.floor(Date.now() / 1000);
    const timeUntilExpiry = payload.exp - now;
    
    if (timeUntilExpiry > 300) { // More than 5 minutes left
      return NextResponse.json({
        message: 'Token still valid, no refresh needed',
        expiresAt: payload.exp * 1000,
        timeUntilExpiry: timeUntilExpiry
      });
    }
    
    // For simple refresh, validate the current session with backend
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    try {
      // Make a userinfo request to validate the admin session
      const response = await fetch(`${backendUrl}/oauth/userinfo`, {
        headers: {
          'Authorization': `Bearer ${payload.sub}`, // Use user ID as simplified token
          'Content-Type': 'application/json'
        }
      });
      
      if (!response.ok) {
        // Admin session is no longer valid on backend
        return NextResponse.json({
          error: 'Admin session expired on backend'
        }, { status: 401 });
      }
      
      // Validate admin permissions are still active
      const userInfo = await response.json();
      
      // Check structured permissions system
      const hasAdminPermissions = userInfo.permissions && userInfo.permissions.some((p: string) => 
        p.includes(':manage') || p.includes(':admin') || p === '*'
      );
      
      if (!hasAdminPermissions) {
        return NextResponse.json({
          error: 'Admin privileges revoked'
        }, { status: 403 });
      }
      
      // Session is still valid, return current token info
      return NextResponse.json({
        message: 'Admin session validated with backend',
        expiresAt: payload.exp * 1000,
        isValid: true,
        permissions: userInfo.permissions || []
      });
      
    } catch (fetchError) {
      console.error('❌ Admin backend session validation failed:', fetchError);
      
      // Return current session info even if backend check failed
      return NextResponse.json({
        message: 'Backend validation failed, using cached admin session',
        expiresAt: payload.exp * 1000,
        warning: 'Backend unreachable'
      });
    }
    
  } catch (error) {
    console.error('❌ Admin session refresh error:', error);
    return NextResponse.json({
      error: 'Failed to refresh admin session'
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