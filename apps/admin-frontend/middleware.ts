import { withAuth } from 'next-auth/middleware';
import { NextResponse } from 'next/server';

export default withAuth(
  function middleware(req) {
    const { pathname } = req.nextUrl;
    const token = req.nextauth.token;

    // Check if user has admin role
    const isAdmin = token?.role === 'admin' || 
                    token?.role === 'system_administrator' ||
                    token?.role === 'super_admin';
    
    if (!isAdmin) {
      const unauthorizedUrl = new URL('/unauthorized', req.url);
      return NextResponse.redirect(unauthorizedUrl);
    }

    // For protected admin routes, check permissions if needed
    const protectedRoutes = ['/admin', '/users', '/iam', '/analytics', '/settings', '/billing', '/modules'];
    const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));
    
    if (isProtectedRoute) {
      // Here you could add additional permission checks if needed
      // For now, we just allow all admin users
      const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
      
      // You could validate specific route access with backend if needed
      // This would require an async call, which middleware doesn't support well
      // So we'll rely on the frontend components to handle specific permission checks
    }

    return NextResponse.next();
  },
  {
    callbacks: {
      authorized: ({ token, req }) => {
        const { pathname } = req.nextUrl;
        
        // Skip auth for public routes
        if (pathname === '/login' || pathname === '/unauthorized' || pathname === '/access-denied') {
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
     * - login (temporary debug - allow login without middleware)
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public|login).*)',
  ],
};