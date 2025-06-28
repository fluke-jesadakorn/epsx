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

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Allow public paths
  if (publicPaths.includes(pathname)) {
    return NextResponse.next();
  }

  // Check for session cookie presence
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
