import { NextRequest, NextResponse } from 'next/server';

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
    '/auth/callback', // OIDC callback
    '/terms',
    '/privacy',
    '/analytics',
  ];
  
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  if (isPublicRoute) {
    return NextResponse.next();
  }
  
  // Check for HTTP-only authentication cookie
  const authCookie = request.cookies.get('auth-token');
  
  if (!authCookie?.value) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
  
  // Note: Detailed permission checking is handled by server components
  // This middleware only ensures basic authentication via HTTP-only cookies
  // The backend validates the JWT token and permissions on each request
  
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