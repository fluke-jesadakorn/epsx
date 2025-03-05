import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const sessionCookie = request.cookies.get('__session')?.value;

  // Protected routes
  const protectedRoutes = ['/home', '/settings'];
  const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));

  // Admin routes
  const isAdminRoute = pathname.startsWith('/admin');

  // Auth routes that should redirect to home if logged in
  const authRoutes = ['/login', '/register'];
  const isAuthRoute = authRoutes.includes(pathname);

  if (isAuthRoute && sessionCookie) {
    return NextResponse.redirect(new URL('/home', request.url));
  }

  if ((isProtectedRoute || isAdminRoute) && !sessionCookie) {
    return NextResponse.redirect(new URL('/login', request.url));
  }

  // Check admin access
  if (isAdminRoute) {
    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/auth/me`, {
        headers: {
          Cookie: `__session=${sessionCookie}`,
        },
        credentials: 'include',
      });

      if (!response.ok) {
        return NextResponse.redirect(new URL('/unauthorized', request.url));
      }

      const data = await response.json();
      if (data.role !== 'admin') {
        return NextResponse.redirect(new URL('/unauthorized', request.url));
      }
    } catch (error) {
      console.error('Error checking admin status:', error);
      return NextResponse.redirect(new URL('/unauthorized', request.url));
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
     */
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
};
