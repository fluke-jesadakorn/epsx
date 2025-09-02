/**
 * Frontend Session API Route
 * OIDC Migration: Handles session management via OIDC-compliant cookies
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET() {
  try {
    const cookieStore = await cookies();
    
    // OIDC Migration: Check for standard OIDC tokens in cookies
    const accessToken = cookieStore.get('access_token')?.value;
    const idToken = cookieStore.get('id_token')?.value;
    const refreshToken = cookieStore.get('refresh_token')?.value;

    console.log('🔍 OIDC Cookie Check:', {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing', 
      refreshToken: refreshToken ? 'present' : 'missing'
    });
    
    // Validate that we have the required OIDC tokens
    if (!accessToken || !idToken) {
      console.log('❌ Missing required OIDC tokens for session');
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'No valid OIDC session found'
      }, { status: 401 });
    }

    // Get user info from backend using Bearer token
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const userInfoResponse = await fetch(`${backendUrl}/oauth/userinfo`, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    });

    if (!userInfoResponse.ok) {
      console.error('❌ Failed to fetch user info:', userInfoResponse.status);
      
      // If access token is expired, try to refresh
      if (userInfoResponse.status === 401 && refreshToken) {
        console.log('🔄 Access token expired, attempting refresh...');
        // This would normally trigger refresh flow, for now return unauthorized
        return NextResponse.json({
          isAuthenticated: false,
          user: null,
          error: 'Access token expired, refresh needed'
        }, { status: 401 });
      }
      
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'Failed to validate session with backend'
      }, { status: 401 });
    }

    const userInfo = await userInfoResponse.json();
    console.log('✅ Successfully retrieved user info from OIDC backend');

    // Return OIDC-compliant session data for client-side hooks
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        id: userInfo.sub,
        email: userInfo.email,
        name: userInfo.name,
        permissions: userInfo.permissions || [],
        platform_context: userInfo.platform_context,
      },
      // Calculate expiry from token (simplified - in production would decode JWT)
      expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour from now
      tokenType: 'oidc'
    });

  } catch (error) {
    console.error('❌ OIDC Session API error:', error);
    
    return NextResponse.json({
      isAuthenticated: false,
      user: null,
      error: 'OIDC session verification failed'
    }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { accessToken, idToken, refreshToken } = body;
    
    // OIDC Migration: Require all three OIDC tokens
    if (!accessToken || !idToken || !refreshToken) {
      return NextResponse.json({
        error: 'All OIDC tokens (accessToken, idToken, refreshToken) required'
      }, { status: 400 });
    }
    
    const response = NextResponse.json({ success: true, tokenType: 'oidc' });
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/'
    };
    
    // Store OIDC tokens in separate HttpOnly cookies
    response.cookies.set('access_token', accessToken, {
      ...cookieOptions,
      maxAge: 60 * 60, // 1 hour
    });
    
    response.cookies.set('id_token', idToken, {
      ...cookieOptions, 
      maxAge: 60 * 60, // 1 hour
    });
    
    response.cookies.set('refresh_token', refreshToken, {
      ...cookieOptions,
      maxAge: 30 * 24 * 60 * 60, // 30 days
    });
    
    // Clean up legacy JWT cookie
    response.cookies.delete('epsx_frontend_jwt');
    
    return response;
    
  } catch (error) {
    console.error('❌ OIDC Session store error:', error);
    return NextResponse.json({
      error: 'Failed to store OIDC session'
    }, { status: 500 });
  }
}

export async function PUT() {
  try {
    const cookieStore = await cookies();
    
    // OIDC Migration: Get refresh token for token refresh
    const refreshToken = cookieStore.get('refresh_token')?.value;
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!refreshToken) {
      return NextResponse.json({
        error: 'No valid refresh token for session refresh'
      }, { status: 401 });
    }
    
    // Use OIDC token refresh endpoint  
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    try {
      console.log('🔄 Refreshing OIDC tokens using refresh token...');
      
      // Call backend OIDC token refresh endpoint
      const response = await fetch(`${backendUrl}/api/v1/oidc/token/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          refresh_token: refreshToken
        })
      });
      
      if (!response.ok) {
        console.error('❌ OIDC token refresh failed:', response.status);
        return NextResponse.json({
          error: 'Refresh token expired or invalid'
        }, { status: 401 });
      }
      
      const tokens = await response.json();
      console.log('✅ Successfully refreshed OIDC tokens');
      
      // Update cookies with new tokens
      const refreshResponse = NextResponse.json({
        message: 'OIDC tokens refreshed successfully',
        expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour from now
        tokenType: 'oidc'
      });
      
      const cookieOptions = {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax' as const,
        path: '/'
      };
      
      // Update access and ID tokens
      refreshResponse.cookies.set('access_token', tokens.access_token, {
        ...cookieOptions,
        maxAge: 60 * 60, // 1 hour
      });
      
      refreshResponse.cookies.set('id_token', tokens.id_token, {
        ...cookieOptions,
        maxAge: 60 * 60, // 1 hour
      });
      
      // Update refresh token if provided (rotation)
      if (tokens.refresh_token) {
        refreshResponse.cookies.set('refresh_token', tokens.refresh_token, {
          ...cookieOptions,
          maxAge: 30 * 24 * 60 * 60, // 30 days
        });
      }
      
      return refreshResponse;
      
    } catch (fetchError) {
      console.error('❌ OIDC token refresh failed:', fetchError);
      return NextResponse.json({
        error: 'Failed to refresh OIDC tokens'
      }, { status: 500 });
    }
    
  } catch (error) {
    console.error('❌ OIDC Session refresh error:', error);
    return NextResponse.json({
      error: 'Failed to refresh OIDC session'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const response = NextResponse.json({ success: true, tokenType: 'oidc' });
    
    // OIDC Migration: Clear all OIDC cookies
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');
    
    // Also clear legacy JWT cookie for backwards compatibility
    response.cookies.delete('epsx_frontend_jwt');
    
    console.log('✅ All OIDC session cookies cleared');
    
    return response;
    
  } catch (error) {
    console.error('❌ OIDC Session clear error:', error);
    return NextResponse.json({
      error: 'Failed to clear OIDC session'
    }, { status: 500 });
  }
}