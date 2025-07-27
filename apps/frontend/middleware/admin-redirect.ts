import { NextRequest, NextResponse } from 'next/server';

/**
 * Handles redirection of admin routes from frontend to admin-frontend
 * This provides seamless migration of admin features
 */
export function adminRedirectMiddleware(request: NextRequest) {
  const { pathname, search } = request.nextUrl;
  
  // Check if this is an admin route
  if (pathname.startsWith('/admin')) {
    // Get the admin frontend URL from environment or default
    const adminFrontendUrl = process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
    
    // Preserve the current path and query parameters
    const redirectUrl = `${adminFrontendUrl}${pathname}${search}`;
    
    // Forward essential cookies to maintain session
    const response = NextResponse.redirect(redirectUrl);
    
    // Copy authentication cookies
    const authToken = request.cookies.get('sess_id');
    const email = request.cookies.get('email');
    const role = request.cookies.get('role');
    
    if (authToken) {
      response.cookies.set('sess_id', authToken.value, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 7, // 7 days
      });
    }
    
    if (email) {
      response.cookies.set('email', email.value, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 7, // 7 days
      });
    }
    
    if (role) {
      response.cookies.set('role', role.value, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 7, // 7 days
      });
    }
    
    return response;
  }
  
  return null; // Continue to next middleware
}
