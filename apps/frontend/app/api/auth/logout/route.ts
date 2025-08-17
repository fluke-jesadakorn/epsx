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

    // Call backend OAuth logout endpoint to properly revoke tokens (if token exists)
    if (accessToken) {
      try {
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
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
          console.log('✅ Frontend: Backend token revocation successful:', result.message);
        } else {
          console.warn('⚠️ Frontend: Backend token revocation failed, but cookies cleared');
        }
      } catch (backendError) {
        console.warn('⚠️ Frontend: Backend token revocation error:', backendError);
        // Continue with cookie clearing even if backend fails
      }
    } else {
      console.log('💡 Frontend: No access token found, skipping backend revocation');
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