import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Handle redirection from /checkout to /payment
  if (pathname === '/checkout') {
    const url = request.nextUrl.clone();
    url.pathname = '/payment';
    return NextResponse.redirect(url);
  }

  // Skip API routes and static files
  if (
    pathname.startsWith('/api/') ||
    pathname.startsWith('/_next/') ||
    pathname.includes('.') // Static files
  ) {
    return NextResponse.next();
  }

  // Temporarily disable auth middleware to debug the google-logging-utils error
  // TODO: Re-enable auth middleware once Edge Runtime compatibility is resolved
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
