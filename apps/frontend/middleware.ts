import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

const SESSION_KEY = '__session';

// List of paths that don't require authentication
const publicPaths = [
  '/',
  '/login',
  '/signup',
  '/register',
  '/privacy',
  '/terms',
  '/reset-password'
];

// List of paths that require session but allow public access
const semiPublicPaths = [
  '/payment',
  '/checkout' // Include checkout for redirection handling
];

export function middleware(request: NextRequest) {
  const { pathname, searchParams } = request.nextUrl;

  // Handle redirection from /checkout to /payment
  if (pathname === '/checkout') {
    const url = request.nextUrl.clone();
    url.pathname = '/payment';
    // Preserve all query parameters
    return NextResponse.redirect(url);
  }

  // Allow public paths
  if (publicPaths.includes(pathname)) {
    return NextResponse.next();
  }

  // Allow semi-public paths but attach session if available
  if (semiPublicPaths.includes(pathname)) {
    const sessionCookie = request.cookies.get(SESSION_KEY);
    const response = NextResponse.next();
    
    if (sessionCookie?.value) {
      response.cookies.set(SESSION_KEY, sessionCookie.value, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax'
      });
    }
    
    return response;
  }

  // Check for session cookie presence for protected routes
  const sessionCookie = request.cookies.get(SESSION_KEY);
  
  if (!sessionCookie?.value) {
    // Redirect to login with return URL
    const searchParams = new URLSearchParams({
      returnUrl: pathname,
    });
    return NextResponse.redirect(new URL(`/login?${searchParams}`, request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    /*
     * Match all paths except:
     * 1. /api routes
     * 2. /_next (Next.js internals)
     * 3. Static files
     */
    '/((?!api|_next/static|_next/image|favicon\\.ico).*)'
  ]
};
