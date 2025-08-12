import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

/**
 * OIDC Logout Route Handler
 * Handles secure logout and token cleanup for frontend users
 * 
 * Features:
 * - Clears all HTTP-only authentication cookies
 * - Logs audit trail for security monitoring
 * - Redirects to backend OIDC logout endpoint for complete session cleanup
 */
export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    
    // Clear all authentication cookies
    cookieStore.delete('auth-token');
    cookieStore.delete('refresh-token');
    cookieStore.delete('id-token');
    
    // Log successful logout
    console.log('🔐 User logout successful', {
      timestamp: new Date().toISOString(),
      client_id: 'epsx-frontend',
      action: 'logout',
      cookies_cleared: ['auth-token', 'refresh-token', 'id-token']
    });

    // Redirect to backend OIDC logout endpoint for complete session cleanup
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const postLogoutRedirectUri = `${process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000'}/login`;
    
    const logoutUrl = `${backendUrl}/oauth/logout?post_logout_redirect_uri=${encodeURIComponent(postLogoutRedirectUri)}&client_id=epsx-frontend`;
    
    return NextResponse.redirect(logoutUrl);

  } catch (error) {
    console.error('Logout error:', error);
    // Even if there's an error, redirect to login for security
    return NextResponse.redirect(new URL('/login?message=logged_out', request.url));
  }
}