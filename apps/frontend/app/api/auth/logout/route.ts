/**
 * Frontend Logout API Route
 * Properly clears JWT cookies and revokes backend tokens
 */
import { NextRequest, NextResponse } from 'next/server';
// Cookie management handled locally

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Processing logout request');

    // Get access token from cookies before clearing
    const accessToken = request.cookies.get('epsx_frontend_jwt')?.value;

    // Create success response
    const response = NextResponse.json({ 
      success: true,
      message: 'Logged out successfully' 
    });

    // Clear all authentication cookies
    response.cookies.delete('epsx_frontend_jwt');

    console.log('✅ Frontend: JWT cookies cleared');

    // Call standard OpenID Connect logout endpoint (RFC compliant)
    if (accessToken) {
      try {
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
        const frontendUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
        
        // Standard OpenID Connect RP-Initiated Logout
        const logoutParams = new URLSearchParams({
          id_token_hint: accessToken,
          post_logout_redirect_uri: `${frontendUrl}/`,
          state: 'frontend-logout'
        });
        
        const logoutResponse = await fetch(`${backendUrl}/oauth/logout?${logoutParams.toString()}`, {
          method: 'GET',
          // Add timeout to prevent hanging
          signal: AbortSignal.timeout(5000),
        });

        if (logoutResponse.ok) {
          console.log('✅ Frontend: Standard OIDC logout successful');
        } else {
          console.warn('⚠️ Frontend: OIDC logout failed, but cookies cleared');
        }
      } catch (backendError) {
        console.warn('⚠️ Frontend: OIDC logout error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Frontend: No access token found, skipping OIDC logout');
    }

    console.log('✅ Frontend: Logout completed successfully');
    return response;

  } catch (error) {
    console.error('❌ Frontend: Logout error:', error);

    // Still try to clear cookies even if there's an error
    const response = NextResponse.json({ 
      success: false,
      error: 'Logout failed',
      message: 'An error occurred during logout'
    }, { status: 500 });

    response.cookies.delete('epsx_frontend_jwt');

    return response;
  }
}

// Also support GET method for simple logout links
export async function GET(request: NextRequest) {
  return POST(request);
}