/**
 * Simplified Admin Frontend Middleware - SharedOpenIDWeb3Provider Compatible
 * Works with OIDC token-based authentication from SharedOpenIDWeb3Provider
 */
import { NextRequest, NextResponse } from 'next/server';

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/auth',
  '/api/auth',
  '/api', // Allow API routes to handle auth themselves
  '/api/proxy', // Allow proxy routes to handle auth themselves
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico',
  '/.well-known', // Chrome DevTools and browser-specific paths
  '/robots.txt',
  '/sitemap.xml',
  '/manifest.json'
]

/**
 *
 * @param request
 */
export async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const startTime = performance.now();

  // Create response with security headers
  const response = NextResponse.next();

  // Add security headers for admin app
  response.headers.set('x-pathname', pathname);
  response.headers.set('x-middleware-timestamp', Date.now().toString());
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-Robots-Tag', 'noindex, nofollow');
  response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
  response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

  // Allow access to public routes
  const isPublicRoute = publicRoutes.some(route =>
    pathname === route || pathname.startsWith(route)
  );

  if (isPublicRoute) {
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());
    return response;
  }

  // All authentication is now handled client-side by SharedOpenIDWeb3Provider
  // Server middleware only adds security headers and allows all requests through

  // Performance tracking
  const elapsedTime = performance.now() - startTime;
  response.headers.set('x-middleware-performance', elapsedTime.toString());

  return response;
}

export const config = {
  matcher: [
    // Enable middleware for all routes except public ones
    '/((?!api/auth|api/public|_next/static|_next/image|favicon.ico|login|auth|unauthorized|access-denied).*)',
  ],
}