import { NextRequest, NextResponse } from 'next/server';
import { getSessionFromCookie } from '@/lib/auth/session';

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
    '/terms',
    '/privacy',
    '/analytics',
    '/api/auth/signin',
    '/api/auth/signout', 
    '/api/auth/callback',
    '/api/auth/session',
  ];
  
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  // Allow access to public routes
  if (isPublicRoute) {
    return NextResponse.next();
  }
  
  try {
    // Check authentication status using manual session cookie
    const sessionCookie = request.cookies.get('epsx-frontend-session')?.value;
    const session = await getSessionFromCookie(sessionCookie);
    const isLoggedIn = session.isLoggedIn && session.user;
    
    // Redirect to login if not authenticated
    if (!isLoggedIn) {
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Allow access to protected routes for authenticated users
    return NextResponse.next();
  } catch (error) {
    console.error('Middleware authentication check failed:', error);
    
    // Redirect to login on error
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
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