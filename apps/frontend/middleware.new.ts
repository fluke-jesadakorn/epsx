import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

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

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Skip middleware for excluded routes and API routes
  if (isRouteType(pathname, 'excluded') || isRouteType(pathname, 'apiRoutes')) {
    return NextResponse.next();
  }
  
  // Handle special redirects
  if (pathname === '/checkout') {
    const url = request.nextUrl.clone();
    url.pathname = '/payment';
    return NextResponse.redirect(url);
  }
  
  // Allow all requests to pass through - authentication will be handled client-side
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
