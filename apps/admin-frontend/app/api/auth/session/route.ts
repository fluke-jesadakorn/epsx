/**
 * Admin Frontend Session API Route  
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

    console.log('🔍 Admin OIDC Cookie Check:', {
      accessToken: accessToken ? 'present' : 'missing',
      idToken: idToken ? 'present' : 'missing', 
      refreshToken: refreshToken ? 'present' : 'missing'
    });
    
    // Validate that we have the required OIDC tokens
    if (!accessToken || !idToken) {
      console.log('❌ Missing required OIDC tokens for admin session');
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'No valid OIDC admin session found'
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
      console.error('❌ Failed to fetch admin user info:', userInfoResponse.status);
      
      // If access token is expired, try to refresh
      if (userInfoResponse.status === 401 && refreshToken) {
        console.log('🔄 Admin access token expired, attempting refresh...');
        // This would normally trigger refresh flow, for now return unauthorized
        return NextResponse.json({
          isAuthenticated: false,
          user: null,
          error: 'Admin access token expired, refresh needed'
        }, { status: 401 });
      }
      
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'Failed to validate admin session with backend'
      }, { status: 401 });
    }

    const userInfo = await userInfoResponse.json();
    
    // ✅ NEW: Validate admin permissions using structured permission system
    const hasAdminAccess = userInfo.permissions?.some((p: string) => 
      p === 'admin:*:*' ||           // Full admin access
      p.startsWith('admin:')         // Any admin-scoped permission
    ) || false;
    
    if (!hasAdminAccess) {
      console.warn('⚠️ User lacks admin permissions for admin frontend', {
        permissions: userInfo.permissions,
        required: 'admin:*:* or admin:{resource}:{action}'
      });
      return NextResponse.json({
        isAuthenticated: false,
        user: null,
        error: 'Insufficient admin permissions'
      }, { status: 403 });
    }
    
    console.log('✅ Successfully retrieved admin user info from OIDC backend');

    // Return OIDC-compliant admin session data
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
      tokenType: 'oidc',
      adminAccess: true
    });

  } catch (error) {
    console.error('❌ Admin OIDC Session API error:', error);
    
    return NextResponse.json({
      isAuthenticated: false,
      user: null,
      error: 'OIDC admin session verification failed'
    }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { accessToken, idToken, refreshToken } = body;
    
    // OIDC Migration: Require all three OIDC tokens for admin access
    if (!accessToken || !idToken || !refreshToken) {
      return NextResponse.json({
        error: 'All OIDC tokens (accessToken, idToken, refreshToken) required for admin access'
      }, { status: 400 });
    }
    
    // Validate admin permissions before storing tokens
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const userInfoResponse = await fetch(`${backendUrl}/oauth/userinfo`, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json'
      }
    });

    if (!userInfoResponse.ok) {
      return NextResponse.json({
        error: 'Invalid access token provided'
      }, { status: 401 });
    }

    const userInfo = await userInfoResponse.json();
    
    // Validate admin permissions
    const hasAdminAccess = userInfo.permissions?.some((p: string) => 
      p === 'admin:*:*' || p.startsWith('admin:')
    ) || false;
    
    if (!hasAdminAccess) {
      return NextResponse.json({
        error: 'Insufficient admin permissions - admin:* required'
      }, { status: 403 });
    }
    
    const response = NextResponse.json({ 
      success: true, 
      tokenType: 'oidc',
      adminAccess: true
    });
    
    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax' as const,
      path: '/'
    };
    
    // Store OIDC tokens in separate HttpOnly cookies for admin access
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
    
    // Clean up legacy admin JWT cookie
    response.cookies.delete('epsx_admin_jwt');
    
    console.log('✅ Admin OIDC tokens stored successfully');
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin OIDC Session store error:', error);
    return NextResponse.json({
      error: 'Failed to store admin OIDC session'
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
        error: 'No valid refresh token for admin session refresh'
      }, { status: 401 });
    }
    
    // Use OIDC token refresh endpoint  
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    
    try {
      console.log('🔄 Refreshing admin OIDC tokens using refresh token...');
      
      // Call backend OIDC token refresh endpoint
      const response = await fetch(`${backendUrl}/oauth/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: new URLSearchParams({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: 'epsx-admin-frontend'
        })
      });
      
      if (!response.ok) {
        console.error('❌ Admin OIDC token refresh failed:', response.status);
        return NextResponse.json({
          error: 'Admin refresh token expired or invalid'
        }, { status: 401 });
      }
      
      const tokens = await response.json();
      console.log('✅ Successfully refreshed admin OIDC tokens');
      
      // Validate admin permissions are still active
      const userInfoResponse = await fetch(`${backendUrl}/oauth/userinfo`, {
        headers: {
          'Authorization': `Bearer ${tokens.access_token}`,
          'Content-Type': 'application/json'
        }
      });
      
      if (userInfoResponse.ok) {
        const userInfo = await userInfoResponse.json();
        
        // Check admin permissions
        const hasAdminAccess = userInfo.permissions?.some((p: string) => 
          p === 'admin:*:*' || p.startsWith('admin:')
        ) || false;
        
        if (!hasAdminAccess) {
          return NextResponse.json({
            error: 'Admin privileges revoked'
          }, { status: 403 });
        }
      }
      
      // Update cookies with new tokens
      const refreshResponse = NextResponse.json({
        message: 'Admin OIDC tokens refreshed successfully',
        expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour from now
        tokenType: 'oidc',
        adminAccess: true
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
      console.error('❌ Admin OIDC token refresh failed:', fetchError);
      return NextResponse.json({
        error: 'Failed to refresh admin OIDC tokens'
      }, { status: 500 });
    }
    
  } catch (error) {
    console.error('❌ Admin OIDC Session refresh error:', error);
    return NextResponse.json({
      error: 'Failed to refresh admin OIDC session'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const response = NextResponse.json({ 
      success: true, 
      tokenType: 'oidc',
      adminAccess: false 
    });
    
    // OIDC Migration: Clear all OIDC cookies for admin session
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');
    
    // Also clear legacy admin JWT cookie for backwards compatibility
    response.cookies.delete('epsx_admin_jwt');
    
    console.log('✅ All admin OIDC session cookies cleared');
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin OIDC Session clear error:', error);
    return NextResponse.json({
      error: 'Failed to clear admin OIDC session'
    }, { status: 500 });
  }
}