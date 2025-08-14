/**
 * Simplified Middleware for Admin Frontend
 * Uses manual session cookie approach that works with session API
 */

import { NextRequest, NextResponse } from 'next/server';
import type { SessionData } from '@/lib/auth/session';
import { getSessionFromCookie } from '@/lib/auth/session';

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/api/auth/callback',
  '/api/auth/login',
  '/api/auth/signin',
  '/api/auth/signout', 
  '/api/auth/session',
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico'
]

// Routes that require specific admin modules
const adminModuleRoutes: Record<string, string> = {
  '/users': 'user_management',
  '/analytics': 'analytics', 
  '/billing': 'billing_admin',
  '/settings': 'system_admin',
  '/permissions': 'permission_admin',
  '/modules': 'module_coordinator'
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  console.log('🔧 MIDDLEWARE: Processing request for:', pathname);
  
  // Add pathname to request headers for server components
  const response = NextResponse.next();
  response.headers.set('x-pathname', pathname);
  
  // Allow access to public routes
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route)
  );
  
  if (isPublicRoute) {
    console.log('🔧 MIDDLEWARE: Public route, allowing access');
    return response;
  }
  
  try {
    // Get session cookie (same approach as session API)
    const sessionCookie = request.cookies.get('epsx-admin-session');
    
    if (!sessionCookie?.value) {
      console.log('🔧 MIDDLEWARE: No session cookie found, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Check session using the same method as session API
    console.log('🔧 MIDDLEWARE: Found session cookie, verifying...');
    const session = getSessionFromCookie(sessionCookie.value);
    
    if (!session.isLoggedIn || !session.user) {
      console.log('🔧 MIDDLEWARE: Invalid session, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    console.log('🔧 MIDDLEWARE: Valid session found for user:', session.user.email);
    
    // Check for admin module requirements
    const requiredModule = Object.entries(adminModuleRoutes).find(([route]) => 
      pathname.startsWith(route)
    )?.[1];
    
    if (requiredModule && session.user) {
      const userAdminModules = session.user.admin_modules || [];
      
      // Check if user has required admin module
      if (!userAdminModules.includes(requiredModule)) {
        console.log('🔧 MIDDLEWARE: User lacks required module:', requiredModule);
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('required_module', requiredModule);
        return NextResponse.redirect(accessDeniedUrl);
      }
    }
    
    console.log('🔧 MIDDLEWARE: Access granted to:', pathname);
    return response;
    
  } catch (error) {
    console.error('❌ MIDDLEWARE: Session error:', error);
    // Redirect to login on session errors
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
}

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