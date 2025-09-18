/**
 * Admin Frontend Logout API Route  
 * Clears JWT cookies and revokes backend tokens
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { URL, URLContext, Service } from '../../../../../../shared/utils/url-resolver';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request');

    // OIDC Migration: Get access token from OIDC cookies
    const cookieStore = await cookies();
    const jwt = cookieStore.get('access_token')?.value;

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'Admin logged out successfully' 
    });

    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    console.log('✅ Admin: OIDC cookies cleared');

    // Call standard OpenID Connect logout endpoint (RFC compliant)
    if (jwt) {
      try {
        const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
        const adminUrl = URL.get(Service.ADMIN, URLContext.SERVER);
        
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

    // OIDC Migration: Clear OIDC tokens on error
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    return response;
  }
}

export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Admin: Processing logout request (GET)');

    // OIDC Migration: Get access token from OIDC cookies
    const cookieStore = await cookies();
    const jwt = cookieStore.get('access_token')?.value;

    // Create redirect response
    const loginUrl = new URL('/login', request.url);
    const response = NextResponse.redirect(loginUrl);

    // OIDC Migration: Clear OIDC tokens instead of legacy JWT
    response.cookies.delete('access_token');
    response.cookies.delete('id_token'); 
    response.cookies.delete('refresh_token');

    console.log('✅ Admin: OIDC cookies cleared, redirecting to login');

    // Call standard OpenID Connect logout endpoint (RFC compliant)
    if (jwt) {
      try {
        const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
        const adminUrl = URL.get(Service.ADMIN, URLContext.SERVER);
        
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
    
    // OIDC Migration: Clear OIDC tokens on error
    response.cookies.delete('access_token');
    response.cookies.delete('id_token');
    response.cookies.delete('refresh_token');

    return response;
  }
}