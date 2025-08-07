import { withAuth } from 'next-auth/middleware';
import { NextResponse } from 'next/server';

export default withAuth(
  function middleware(req) {
    const { pathname } = req.nextUrl;
    const token = req.nextauth.token;

    // Skip middleware for error pages to prevent redirect loops
    if (pathname === '/unauthorized' || pathname === '/access-denied' || pathname === '/login') {
      return NextResponse.next();
    }

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

    // Check if user has admin role - expanded to match auth.ts
    const isAdmin = token?.role === 'admin' || 
                    token?.role === 'system_administrator' ||
                    token?.role === 'super_admin' ||
                    token?.role === 'moderator';
    
    if (!isAdmin) {
      // Prevent redirect loops by checking if we're already redirecting to unauthorized
      if (pathname !== '/unauthorized') {
        const unauthorizedUrl = new URL('/unauthorized', req.url);
        return NextResponse.redirect(unauthorizedUrl);
      }
    }

    return NextResponse.next();
  },
  {
    callbacks: {
      authorized: ({ token, req }) => {
        const { pathname } = req.nextUrl;
        
        // Skip auth for public routes - prevent redirect loops
        const publicRoutes = ['/login', '/unauthorized', '/access-denied'];
        if (publicRoutes.includes(pathname)) {
          return true;
        }

        // Require authentication for all other routes
        return !!token;
      },
    },
  }
);

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