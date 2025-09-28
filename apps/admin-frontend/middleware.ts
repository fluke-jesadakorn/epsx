/**
 * Simplified Admin Frontend Middleware - Backend-Only Permission Architecture
 * Only handles authentication, all permission validation moved to backend
 */
import { NextRequest, NextResponse } from 'next/server';
import { env } from '../../shared/env/schema';

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/api/auth',
  '/api/v1', // Allow API routes to handle auth themselves
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

export async function middleware(request: NextRequest) {
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
  
  try {
    console.log(`🔍 Admin middleware: Checking authentication for ${pathname}`);
    
    // Check for authentication using unified auth
    const { UnifiedAuth } = await import('@/lib/auth/unified-auth');
    const session = await UnifiedAuth.getSession();
    
    if (!session.isAuthenticated) {
      console.log(`🚫 Admin middleware: User not authenticated for ${pathname}`);
      return redirectToLogin(request);
    }
    
    if (!session.hasAdminAccess) {
      console.log(`🚫 Admin middleware: User lacks admin access for ${pathname}`);
      const accessDeniedUrl = new URL('/access-denied', request.url);
      accessDeniedUrl.searchParams.set('reason', 'no-admin-permissions');
      return NextResponse.redirect(accessDeniedUrl);
    }
    
    console.log(`✅ Admin middleware: Authenticated user accessing ${pathname}`);
    
    // Performance tracking
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin middleware error:', error);
    return redirectToLogin(request);
  }
}

/**
 * Create redirect response to login page
 */
function redirectToLogin(request: NextRequest): NextResponse {
  const adminUrl = `${request.nextUrl.protocol}//${request.nextUrl.host}`;
  const returnUrl = `${request.nextUrl.pathname}${request.nextUrl.search}`;
  
  const loginUrl = new URL('/login', adminUrl);
  loginUrl.searchParams.set('return_url', returnUrl);
  loginUrl.searchParams.set('reason', 'no-session');
  
  console.log(`🔄 Admin middleware: Redirecting to login: ${loginUrl.toString()}`);
  const redirect = NextResponse.redirect(loginUrl.toString());
  
  // Clear authentication cookies
  redirect.cookies.delete('access_token');
  redirect.cookies.delete('id_token');
  redirect.cookies.delete('refresh_token');
  
  return redirect;
}

export const config = {
  matcher: [
    // Enable middleware for all routes except public ones
    '/((?!api/auth|api/public|_next/static|_next/image|favicon.ico|login|unauthorized|access-denied).*)',
  ],
}