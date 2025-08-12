import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

/**
 * Admin OIDC Logout Route Handler
 * Handles secure logout and token cleanup for administrators
 * 
 * Features:
 * - Clears all HTTP-only admin cookies
 * - Logs audit trail for security monitoring
 * - Redirects to backend OIDC logout endpoint for complete session cleanup
 */
export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    
    // Clear all admin authentication cookies
    cookieStore.delete('admin-auth-token');
    cookieStore.delete('admin-refresh-token');
    cookieStore.delete('admin-id-token');
    
    // Log successful admin logout
    console.log('🔐 Admin logout successful', {
      timestamp: new Date().toISOString(),
      client_id: 'epsx-admin',
      action: 'logout',
      cookies_cleared: ['admin-auth-token', 'admin-refresh-token', 'admin-id-token']
    });

    // Redirect to backend OIDC logout endpoint for complete session cleanup
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    const postLogoutRedirectUri = `${process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001'}/login`;
    
    const logoutUrl = `${backendUrl}/oauth/logout?post_logout_redirect_uri=${encodeURIComponent(postLogoutRedirectUri)}&client_id=epsx-admin`;
    
    return NextResponse.redirect(logoutUrl);

  } catch (error) {
    console.error('Admin logout error:', error);
    // Even if there's an error, redirect to login for security
    return NextResponse.redirect(new URL('/login?message=logged_out', request.url));
  }
}