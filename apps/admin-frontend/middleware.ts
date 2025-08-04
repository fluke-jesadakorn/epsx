import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

const AUTH_COOKIE_NAME = 'admin_bearer_token';

// Combined validation to prevent double backend calls using bearer token
async function validateSessionAndAccess(request: NextRequest, route?: string) {
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
  
  // Get bearer token from HTTP-only cookie
  const token = request.cookies.get(AUTH_COOKIE_NAME)?.value;
  
  if (!token) {
    return { valid: false, allowed: false, authData: null };
  }
  
  try {
    // Use validate-access endpoint which also validates session
    const response = await fetch(`${backendUrl}/api/v1/auth/validate-access`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        route: route || request.nextUrl.pathname,
        method: 'GET',
        app_type: 'admin',
      }),
    });

    if (!response.ok) {
      return { valid: false, allowed: false, authData: null };
    }

    const result = await response.json();
    return {
      valid: true,
      allowed: result.allowed || false,
      authData: result.user_data || result,
    };
  } catch (error) {
    console.error('Combined validation failed:', error);
    return { valid: false, allowed: false, authData: null };
  }
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Skip auth for public routes
  if (pathname === '/login' || pathname === '/unauthorized' || pathname === '/access-denied' || pathname === '/api' || pathname.startsWith('/_next') || pathname.startsWith('/public')) {
    return NextResponse.next();
  }

  // Combined session and access validation in single call
  const validation = await validateSessionAndAccess(request, pathname);
  
  if (!validation.valid) {
    // No valid session, redirect to login
    const loginUrl = new URL('/login', request.url);
    return NextResponse.redirect(loginUrl);
  }

  // Check if user has admin role
  const authData = validation.authData;
  const isAdmin = authData?.role === 'admin' || 
                  authData?.role === 'system_administrator' ||
                  authData?.role === 'super_admin';
  
  if (!isAdmin) {
    const unauthorizedUrl = new URL('/unauthorized', request.url);
    return NextResponse.redirect(unauthorizedUrl);
  }

  // For protected admin routes, check if access is allowed
  const protectedRoutes = ['/admin', '/users', '/iam', '/analytics', '/settings', '/billing', '/modules'];
  const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));
  
  // Bypass route validation for super_admin users during development
  if (isProtectedRoute && !validation.allowed && authData?.role !== 'super_admin') {
    const accessDeniedUrl = new URL('/access-denied', request.url);
    return NextResponse.redirect(accessDeniedUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public (public assets)
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};