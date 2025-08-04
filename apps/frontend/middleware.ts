import { withAuth } from 'next-auth/middleware';
import { NextResponse } from 'next/server';

export default withAuth(
  function middleware(req) {
    const { pathname } = req.nextUrl;
    const token = req.nextauth.token;

    // For premium routes, check subscription tier
    const premiumRoutes = ['/analytics', '/trading'];
    const isPremiumRoute = premiumRoutes.some(route => pathname.startsWith(route));
    
    if (isPremiumRoute) {
      const isPremium = token?.subscription_tier && 
        ['premium', 'enterprise', 'platinum', 'gold'].includes(token.subscription_tier.toLowerCase());
      
      if (!isPremium) {
        const upgradeUrl = new URL('/payment', req.url);
        return NextResponse.redirect(upgradeUrl);
      }
    }

    // For admin routes, check admin role
    if (pathname.startsWith('/admin')) {
      const isAdmin = token?.role === 'admin' || token?.role === 'system_administrator';
      if (!isAdmin) {
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
        
        // Public routes that don't require authentication
        const publicRoutes = ['/', '/login', '/register', '/forgot-password', '/reset-password', '/verify-email', '/access-denied', '/unauthorized'];
        const isPublicRoute = publicRoutes.includes(pathname);
        
        if (isPublicRoute) {
          return true;
        }

        // Protected routes that require authentication
        const protectedRoutes = ['/dashboard', '/analytics', '/my-data', '/trading', '/settings', '/payment'];
        const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));
        
        if (isProtectedRoute) {
          return !!token;
        }

        // Default: allow access
        return true;
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
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};