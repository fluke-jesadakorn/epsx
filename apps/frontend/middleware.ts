import { NextRequest, NextResponse } from 'next/server';
import { getToken } from 'next-auth/jwt';

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
    '/api/auth',
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
  
  // Get JWT token (Edge Runtime compatible)
  const token = await getToken({ 
    req: request, 
    secret: process.env.NEXTAUTH_SECRET 
  });
  
  if (!token?.id) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
  
  // For now, let server components handle detailed permission checking
  // This middleware only ensures basic authentication
  
  // Routes that require specific roles (basic check)
  if (pathname.startsWith('/admin')) {
    const isAdmin = token.role === 'admin' || 
                   token.role === 'system_administrator' || 
                   token.role === 'super_admin';
    
    if (!isAdmin) {
      const accessDeniedUrl = new URL('/access-denied', request.url);
      accessDeniedUrl.searchParams.set('reason', 'Admin access required');
      return NextResponse.redirect(accessDeniedUrl);
    }
  }
  
  // For premium routes, check subscription (basic check)
  const premiumRoutes = ['/trading'];
  const isPremiumRoute = premiumRoutes.some(route => pathname.startsWith(route));
  
  if (isPremiumRoute) {
    const isSuperAdmin = token.role === 'system_administrator' || 
                        token.role === 'admin' || 
                        token.role === 'super_admin';
    
    if (!isSuperAdmin) {
      const isPremium = token.subscription_tier && 
        ['premium', 'enterprise', 'platinum', 'gold'].includes(
          token.subscription_tier.toLowerCase()
        );
      
      if (!isPremium) {
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('reason', 'Premium subscription required');
        return NextResponse.redirect(accessDeniedUrl);
      }
    }
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
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};