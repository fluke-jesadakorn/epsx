import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Handle redirection from /checkout to /payment
  if (pathname === '/checkout') {
    const url = request.nextUrl.clone();
    url.pathname = '/payment';
    // Preserve all query parameters
    return NextResponse.redirect(url);
  }

  // For client-side auth, let pages handle their own authentication logic
  // Middleware will only handle route redirects, not authentication
  console.log('Middleware: Allowing access to:', pathname);
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
