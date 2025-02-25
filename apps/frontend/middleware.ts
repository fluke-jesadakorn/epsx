import { NextRequest, NextResponse } from "next/server";
import type { UserSession } from "@/app/api/auth/session/route";

const publicRoutes = [
  "/",
  "/login",
  "/register",
  "/confirmation-pending",
  "/terms",
  "/coming-soon",
  "/unauthorized",
];

const protectedRoutes = [
  "/home",
  "/settings",
  "/admin",
  "/news",
  "/ranking",
  "/services",
  "/developer",
];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Allow public routes
  if (publicRoutes.some(route => pathname.startsWith(route))) {
    return NextResponse.next();
  }

  // Check API routes
  if (pathname.startsWith('/api')) {
    return NextResponse.next();
  }

  // Check protected routes
  const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));
  if (!isProtectedRoute) {
    return NextResponse.next();
  }

  try {
    const response = await fetch(`${request.nextUrl.origin}/api/auth/session`);
    const { user } = (await response.json()) as UserSession;

    if (!user) {
      return NextResponse.redirect(new URL('/login', request.url));
    }

    // Admin route check
    if (pathname.startsWith('/admin') && !user.customClaims?.roles?.includes('admin')) {
      return NextResponse.redirect(new URL('/unauthorized', request.url));
    }

    return NextResponse.next();
  } catch (error) {
    console.error('Middleware error:', error);
    return NextResponse.redirect(new URL('/login', request.url));
  }
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public (public files)
     */
    '/((?!_next/static|_next/image|favicon.ico|public).*)',
  ],
};
