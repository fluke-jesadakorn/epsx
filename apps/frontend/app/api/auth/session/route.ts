/**
 * Frontend Session API Route
 * Handles JWT session management via secure cookies
 */
import { NextRequest, NextResponse } from 'next/server';
import { verifyJWTFromCookies } from '@/lib/server/jwt';
// Cookie management is now handled locally

export async function GET() {
  try {
    // Get JWT payload from cookie
    const payload = await verifyJWTFromCookies();
    
    if (!payload) {
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
        id: payload.sub,
        email: payload.email,
        name: payload.name,
        role: payload.role,
        permissions: payload.permissions,
        package_tier: payload.package_tier,
        firebase_uid: payload.firebase_uid,
      },
      expiresAt: payload.exp * 1000, // Convert to milliseconds
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
    
    const response = NextResponse.json({ success: true });
    
    // Store JWT in secure cookie
    response.cookies.set('epsx_frontend_jwt', accessToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 24 * 60 * 60, // 24 hours
      path: '/'
    });
    
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
    // Get current JWT payload to check if refresh is needed
    const payload = await verifyJWTFromCookies();
    
    if (!payload) {
      return NextResponse.json({
        error: 'No valid session to refresh'
      }, { status: 401 });
    }
    
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
    
    // For simple refresh, we'll validate the current session with backend
    // In a full implementation, this would use refresh tokens
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    try {
      // Make a simple userinfo request to validate the session is still active
      const response = await fetch(`${backendUrl}/oauth/userinfo`, {
        headers: {
          'Authorization': `Bearer ${payload.sub}`, // Use user ID as simplified token
          'Content-Type': 'application/json'
        }
      });
      
      if (!response.ok) {
        // Session is no longer valid on backend
        return NextResponse.json({
          error: 'Session expired on backend'
        }, { status: 401 });
      }
      
      // Session is still valid, return current token info
      return NextResponse.json({
        message: 'Session validated with backend',
        expiresAt: payload.exp * 1000,
        isValid: true
      });
      
    } catch (fetchError) {
      console.error('❌ Backend session validation failed:', fetchError);
      
      // Return current session info even if backend check failed
      return NextResponse.json({
        message: 'Backend validation failed, using cached session',
        expiresAt: payload.exp * 1000,
        warning: 'Backend unreachable'
      });
    }
    
  } catch (error) {
    console.error('❌ Session refresh error:', error);
    return NextResponse.json({
      error: 'Failed to refresh session'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const response = NextResponse.json({ success: true });
    
    // Clear all auth cookies
    response.cookies.delete('epsx_frontend_jwt');
    
    return response;
    
  } catch (error) {
    console.error('❌ Session clear error:', error);
    return NextResponse.json({
      error: 'Failed to clear session'
    }, { status: 500 });
  }
}