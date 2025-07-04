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

  // Check for session cookie on login page - if user has session, redirect to return URL or dashboard
  if (pathname === '/login') {
    const sessionCookie = request.cookies.get(SESSION_KEY);
    if (sessionCookie?.value) {
      // Do not validate token in middleware to avoid using Firebase Admin in Edge runtime
      // Just check if token exists and is not obviously invalid - detailed validation happens server-side
      if (sessionCookie.value.length > 10) { // Basic sanity check
        const returnUrl = searchParams.get('returnUrl') || '/dashboard';
        // Check if this is a post-login redirect to avoid loops
        if (searchParams.get('postLogin') === 'true') {
          return NextResponse.next();
        }
        return NextResponse.redirect(new URL(returnUrl, request.url));
      }
    }
    return NextResponse.next();
  }

  // Allow other public paths
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
        secure: true, // Always use secure cookies; adjust if needed for local development
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
