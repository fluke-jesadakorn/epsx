import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

/**
 * Server-side logout endpoint
 * Clears session cookies and optionally calls backend logout
 */
export async function POST(request: NextRequest) {
  const cookieStore = await cookies();
  
  try {
    // Get current session data
    const sessionCookie = cookieStore.get('admin_session');
    const accessToken = cookieStore.get('admin_access_token');
    
    // Call backend logout endpoint if we have a token
    if (accessToken?.value) {
      await logoutFromBackend(accessToken.value);
    }
    
    // Clear all auth-related cookies
    const cookiesToClear = [
      'admin_session',
      'admin_access_token',
      'admin_refresh_token',
      'oidc_state',
      'oidc_nonce',
      'oidc_callback_url'
    ];
    
    const isProduction = process.env.NODE_ENV === 'production';
    
    cookiesToClear.forEach(cookieName => {
      cookieStore.set(cookieName, '', {
        httpOnly: true,
        secure: isProduction,
        sameSite: 'lax',
        expires: new Date(0), // Set expiry to past date
        path: '/',
        domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
      });
    });
    
    console.log('✅ Admin logout successful:', {
      timestamp: new Date().toISOString(),
      sessionData: sessionCookie ? JSON.parse(sessionCookie.value) : null
    });
    
    return NextResponse.json({ success: true, message: 'Logged out successfully' });
    
  } catch (error) {
    console.error('🚨 Logout Error:', error);
    
    // Still clear cookies even if backend call fails
    const cookiesToClear = [
      'admin_session',
      'admin_access_token', 
      'admin_refresh_token'
    ];
    
    const isProduction = process.env.NODE_ENV === 'production';
    
    cookiesToClear.forEach(cookieName => {
      cookieStore.set(cookieName, '', {
        httpOnly: true,
        secure: isProduction,
        sameSite: 'lax',
        expires: new Date(0),
        path: '/',
        domain: isProduction ? process.env.COOKIE_DOMAIN : undefined,
      });
    });
    
    return NextResponse.json(
      { success: false, message: 'Logout completed with warnings' },
      { status: 200 } // Still return 200 since cookies were cleared
    );
  }
}

/**
 * GET handler for logout (redirect-based logout)
 */
export async function GET(request: NextRequest) {
  // Perform logout
  await POST(request);
  
  // Redirect to login page
  const redirectUrl = `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login`;
  return NextResponse.redirect(redirectUrl);
}

/**
 * Call backend logout endpoint
 */
async function logoutFromBackend(accessToken: string): Promise<void> {
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
  
  try {
    const response = await fetch(`${backendUrl}/oauth/revoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        token: accessToken,
        token_type_hint: 'access_token',
      }).toString(),
    });
    
    if (!response.ok) {
      console.warn(`Backend logout warning: ${response.status} ${response.statusText}`);
    }
  } catch (error) {
    console.warn('Backend logout failed:', error);
    // Don't throw - we still want to clear local cookies
  }
}