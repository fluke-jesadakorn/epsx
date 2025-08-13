/**
 * NextAuth.js Middleware for Admin Frontend with Admin Module Protection
 * Replaces custom auth middleware with NextAuth.js auth wrapper
 */

import { NextResponse } from 'next/server';
import { auth } from '@/lib/auth';

// Routes that require specific admin modules
const adminModuleRoutes: Record<string, string> = {
  '/users': 'user_operations',
  '/analytics': 'analytics_specialist', 
  '/billing': 'billing_admin',
  '/settings': 'system_admin',
  '/permissions': 'permission_admin',
  '/modules': 'module_coordinator'
}

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/auth/callback',
  '/auth/error',
  '/unauthorized',
  '/access-denied',
  '/api/auth',
  '/_next',
  '/favicon.ico'
]

export default auth((request) => {
  const { pathname } = request.nextUrl;
  const isLoggedIn = !!request.auth;
  
  // Add pathname to request headers for server components
  const response = NextResponse.next();
  response.headers.set('x-pathname', pathname);
  
  // Allow access to public routes
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route)
  );
  
  if (isPublicRoute) {
    return response;
  }
  
  // Redirect to login if not authenticated
  if (!isLoggedIn) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
  
  // Check for admin module requirements
  const requiredModule = Object.entries(adminModuleRoutes).find(([route]) => 
    pathname.startsWith(route)
  )?.[1];
  
  if (requiredModule && request.auth?.user) {
    const userAdminModules = (request.auth.user as any).admin_modules as string[] || [];
    
    // Check if user has required admin module
    if (!userAdminModules.includes(requiredModule)) {
      const accessDeniedUrl = new URL('/access-denied', request.url);
      accessDeniedUrl.searchParams.set('required_module', requiredModule);
      return NextResponse.redirect(accessDeniedUrl);
    }
  }
  
  // Allow access to protected routes for authenticated admin users
  return response;
}) as any;

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api/public (public API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api/public|_next/static|_next/image|favicon.ico).*)',
  ],
}