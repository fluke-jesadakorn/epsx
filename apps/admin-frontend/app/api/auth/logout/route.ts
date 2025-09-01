/**
 * Admin Frontend Logout API Route  
 * Clears JWT cookies and revokes backend tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request');

    // Get access token from cookies before clearing (use admin-specific cookie)
    const cookieStore = await cookies();
    const jwt = cookieStore.get('epsx_admin_jwt')?.value || cookieStore.get('epsx_jwt')?.value;

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'Admin logged out successfully' 
    });

    // Clear authentication cookies (both admin and standard)
    response.cookies.delete('epsx_admin_jwt');
    response.cookies.delete('epsx_jwt');

    console.log('✅ Admin: JWT cookies cleared');

    // Call standard OpenID Connect logout endpoint (RFC compliant)
    if (jwt) {
      try {
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
        const adminUrl = process.env.NEXT_PUBLIC_APP_URL || process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
        
        // Standard OpenID Connect RP-Initiated Logout
        const logoutParams = new URLSearchParams({
          id_token_hint: jwt,
          post_logout_redirect_uri: `${adminUrl}/login`,
          state: 'admin-logout'
        });
        
        const logoutResponse = await fetch(`${backendUrl}/oauth/logout?${logoutParams.toString()}`, {
          method: 'GET',
          // Add timeout to prevent hanging
          signal: AbortSignal.timeout(5000),
        });

        if (logoutResponse.ok) {
          console.log('✅ Admin: Standard OIDC logout successful');
        } else {
          console.warn('⚠️ Admin: OIDC logout failed, but cookies cleared');
        }
      } catch (backendError) {
        console.warn('⚠️ Admin: OIDC logout error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Admin: No access token found, skipping OIDC logout');
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

    response.cookies.delete('epsx_jwt');

    return response;
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request (GET)');

    // Get access token from cookies before clearing (use admin-specific cookie)
    const cookieStore = await cookies();
    const jwt = cookieStore.get('epsx_admin_jwt')?.value || cookieStore.get('epsx_jwt')?.value;

    // Create redirect response
    const loginUrl = new URL('/login', request.url);
    const response = NextResponse.redirect(loginUrl);

    // Clear authentication cookies (both admin and standard)
    response.cookies.delete('epsx_admin_jwt');
    response.cookies.delete('epsx_jwt');

    console.log('✅ Admin: JWT cookies cleared, redirecting to login');

    // Call standard OpenID Connect logout endpoint (RFC compliant)
    if (jwt) {
      try {
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
        const adminUrl = process.env.NEXT_PUBLIC_APP_URL || process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001';
        
        // Standard OpenID Connect RP-Initiated Logout
        const logoutParams = new URLSearchParams({
          id_token_hint: jwt,
          post_logout_redirect_uri: `${adminUrl}/login`,
          state: 'admin-logout'
        });
        
        await fetch(`${backendUrl}/oauth/logout?${logoutParams.toString()}`, {
          method: 'GET',
          signal: AbortSignal.timeout(5000),
        });
        console.log('✅ Admin: Standard OIDC logout successful during GET');
      } catch (backendError) {
        console.warn('⚠️ Admin: OIDC logout error during GET:', backendError);
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
    
    response.cookies.delete('epsx_jwt');

    return response;
  }
}