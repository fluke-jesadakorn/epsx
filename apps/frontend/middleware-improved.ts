import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

const SESSION_KEY = '__session';

// Configuration for different route types
const routeConfig = {
  // Routes that don't require authentication
  public: [
    '/',
    '/login',
    '/signup',
    '/register', 
    '/privacy',
    '/terms',
    '/reset-password',
    '/verify-email',
  ],
  
  // Routes that should redirect authenticated users
  guestOnly: [
    '/login',
    '/signup',
    '/register',
  ],
  
  // Routes that allow both authenticated and unauthenticated users
  optional: [
    '/',
    '/privacy',
    '/terms',
  ],
  
  // API routes that should pass through
  apiRoutes: [
    '/api',
  ],
  
  // Static assets and internal Next.js routes
  excluded: [
    '/_next',
    '/favicon.ico',
    '/robots.txt',
    '/sitemap.xml',
  ],
};

function isRouteType(pathname: string, routeType: keyof typeof routeConfig): boolean {
  const routes = routeConfig[routeType];
  return routes.some(route => {
    if (route.endsWith('*')) {
      return pathname.startsWith(route.slice(0, -1));
    }
    return pathname === route || pathname.startsWith(route + '/');
  });
}

function hasValidSession(request: NextRequest): boolean {
  const sessionCookie = request.cookies.get(SESSION_KEY);
  
  if (!sessionCookie?.value) {
    return false;
  }
  
  // Basic validation - check if token looks like a JWT
  const token = sessionCookie.value;
  const parts = token.split('.');
  
  // JWT should have 3 parts
  if (parts.length !== 3) {
    return false;
  }
  
  // Additional basic checks could be added here
  return token.length > 100; // Minimum reasonable token length
}

export function middleware(request: NextRequest) {
  const { pathname, searchParams } = request.nextUrl;
  
  // Skip middleware for excluded routes
  if (isRouteType(pathname, 'excluded') || isRouteType(pathname, 'apiRoutes')) {
    return NextResponse.next();
  }
  
  // Handle special redirects
  if (pathname === '/checkout') {
    const url = request.nextUrl.clone();
    url.pathname = '/payment';
    return NextResponse.redirect(url);
  }
  
  const hasSession = hasValidSession(request);
  
  // Handle guest-only routes (login, signup)
  if (isRouteType(pathname, 'guestOnly')) {
    if (hasSession) {
      // User has session but trying to access guest-only page
      const returnUrl = searchParams.get('returnUrl') || '/dashboard';
      
      // Prevent redirect loops
      if (searchParams.get('postLogin') === 'true') {
        return NextResponse.next();
      }
      
      return NextResponse.redirect(new URL(returnUrl, request.url));
    }
    return NextResponse.next();
  }
  
  // Handle public/optional routes
  if (isRouteType(pathname, 'public') || isRouteType(pathname, 'optional')) {
    return NextResponse.next();
  }
  
  // All other routes require authentication
  if (!hasSession) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('returnUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
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
     * - public files (robots.txt, sitemap.xml, etc.)
     */
    '/((?!api|_next/static|_next/image|favicon\\.ico|robots\\.txt|sitemap\\.xml).*)',
  ],
};
