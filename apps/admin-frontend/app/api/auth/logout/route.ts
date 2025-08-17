/**
 * Admin Frontend Logout API Route
 * Properly clears JWT cookies and revokes backend tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { createCookieManager } from '@epsx/auth-shared';
import { env } from '../../../config/env';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request');

    // Create cookie manager for admin app
    const cookieManager = createCookieManager('admin');

    // Get access token before clearing
    const accessToken = await cookieManager.getAccessToken();

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'Admin logged out successfully' 
    });

    // Clear all authentication cookies
    cookieManager.clearAllCookies(response);

    console.log('✅ Admin: JWT cookies cleared');

    // Call backend OAuth logout endpoint to properly revoke tokens (if token exists)
    if (accessToken) {
      try {
        const backendUrl = env.NEXT_PUBLIC_BACKEND_URL || env.getBackendUrl();
        const logoutResponse = await fetch(`${backendUrl}/oauth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
          },
          // Add timeout to prevent hanging
          signal: AbortSignal.timeout(5000),
        });

        if (logoutResponse.ok) {
          const result = await logoutResponse.json();
          console.log('✅ Admin: Backend token revocation successful:', result.message);
        } else {
          console.warn('⚠️ Admin: Backend token revocation failed, but cookies cleared');
        }
      } catch (backendError) {
        console.warn('⚠️ Admin: Backend token revocation error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Admin: No access token found, skipping backend revocation');
    }

    console.log('✅ Admin: Logout completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Admin: Logout error:', error);

    // Still try to clear cookies even if there's an error
    const response = NextResponse.json({ 
      success: false,
      error: 'Logout failed',
      message: 'An error occurred during admin logout'
    }, { status: 500 });

    const cookieManager = createCookieManager('admin');
    cookieManager.clearAllCookies(response);

    return response;
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request (GET)');

    // Create cookie manager for admin app
    const cookieManager = createCookieManager('admin');

    // Get access token before clearing
    const accessToken = await cookieManager.getAccessToken();

    // Create redirect response
    const loginUrl = new URL('/login', request.url);
    const response = NextResponse.redirect(loginUrl);

    // Clear all authentication cookies
    cookieManager.clearAllCookies(response);

    console.log('✅ Admin: JWT cookies cleared, redirecting to login');

    // Call backend OAuth logout endpoint to revoke tokens (if token exists)
    if (accessToken) {
      try {
        const backendUrl = env.NEXT_PUBLIC_BACKEND_URL || env.getBackendUrl();
        await fetch(`${backendUrl}/oauth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
          },
          signal: AbortSignal.timeout(5000),
        });
        console.log('✅ Admin: Backend token revocation successful during GET');
      } catch (backendError) {
        console.warn('⚠️ Admin: Backend token revocation error during GET:', backendError);
        // Continue with redirect even if backend fails
      }
    }

    return response;

  } catch (error) {
    console.error('❌ Admin: Logout error (GET):', error);
    
    // Still redirect to login on error but clear cookies
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('error', 'logout_error');
    const response = NextResponse.redirect(loginUrl);

    const cookieManager = createCookieManager('admin');
    cookieManager.clearAllCookies(response);

    return response;
  }
}