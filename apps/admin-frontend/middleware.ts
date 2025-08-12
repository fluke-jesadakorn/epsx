import { NextRequest, NextResponse } from 'next/server';

export default function middleware(req: NextRequest) {
  const { pathname } = req.nextUrl;

  // Public routes that don't require authentication
  const publicRoutes = ['/login', '/unauthorized', '/access-denied', '/auth/callback', '/auth/logout'];
  
  if (publicRoutes.includes(pathname)) {
    return NextResponse.next();
  }

  // Check for admin authentication token in sessionStorage (client-side will handle this)
  // For now, we'll let the route components handle authentication checks
  // since OIDC tokens are stored in sessionStorage and not accessible in middleware

  // Redirect /admin to dashboard
  if (pathname === '/admin') {
    const redirectUrl = new URL('/', req.url);
    return NextResponse.redirect(redirectUrl);
  }

  // Legacy route redirects to unified structure
  if (pathname === '/iam') {
    const redirectUrl = new URL('/users', req.url);
    return NextResponse.redirect(redirectUrl);
  }
  
  if (pathname === '/users/permissions' || pathname === '/users/roles') {
    // Redirect to unified user list for now, could be enhanced with user context later
    const redirectUrl = new URL('/users', req.url);
    return NextResponse.redirect(redirectUrl);
  }
  
  if (pathname === '/permission-profiles/assign') {
    const redirectUrl = new URL('/users', req.url);
    return NextResponse.redirect(redirectUrl);
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
     * - login (login page should be accessible without auth)
     * - unauthorized (error pages)
     * - access-denied (error pages)
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public|login|unauthorized|access-denied).*)',
  ],
};