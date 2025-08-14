/**
 * Enhanced JWT Middleware for Frontend (Trading Platform)
 * Uses JWT cookie verification with security headers
 */
import { NextRequest, NextResponse } from 'next/server';
import { verifyJWT } from '@epsx/auth-shared';

export default async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Public routes that don't require authentication
  const publicRoutes = [
    '/',
    '/login',
    '/register', 
    '/forgot-password',
    '/reset-password',
    '/verify-email',
    '/access-denied',
    '/unauthorized',
    '/terms',
    '/privacy',
    '/analytics',
    '/api/auth/signin',
    '/api/auth/signout', 
    '/api/auth/logout',
    '/api/auth/callback',
    '/api/auth/session',
  ];
  
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  // Create response with security headers
  const response = NextResponse.next();
  
  // Add security headers
  response.headers.set('x-pathname', pathname);
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  
  // Allow access to public routes
  if (isPublicRoute) {
    return response;
  }
  
  try {
    // Get JWT token from frontend-specific httpOnly cookie
    const jwtToken = request.cookies.get('epsx_frontend_jwt')?.value;
    
    // Redirect to login if no token
    if (!jwtToken) {
      console.log('🔓 Frontend middleware: No JWT token found, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Verify JWT token
    const payload = await verifyJWT(jwtToken);
    
    // Redirect to login if invalid token
    if (!payload) {
      console.log('🔓 Frontend middleware: Invalid JWT token, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Add user info to headers for server components (non-sensitive data only)
    response.headers.set('x-user-id', payload.sub);
    response.headers.set('x-user-role', payload.role);
    response.headers.set('x-user-package-tier', payload.package_tier || 'FREE');
    
    console.log(`🔐 Frontend middleware: Authenticated user ${payload.email} accessing ${pathname}`);
    
    // Allow access to protected routes for authenticated users
    return response;
    
  } catch (error) {
    console.error('❌ Frontend middleware JWT verification failed:', error);
    
    // Redirect to login on error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
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