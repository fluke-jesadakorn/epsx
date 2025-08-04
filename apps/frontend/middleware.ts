import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

const AUTH_COOKIE_NAME = 'frontend_bearer_token';

// Simple backend-based auth middleware for main frontend using bearer tokens
async function validateWithBackend(request: NextRequest) {
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
  
  // Get bearer token from HTTP-only cookie
  const token = request.cookies.get(AUTH_COOKIE_NAME)?.value;
  
  if (!token) {
    return null;
  }
  
  try {
    const response = await fetch(`${backendUrl}/api/v1/auth/validate-session`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({ app_type: 'frontend' }),
    });

    if (!response.ok) {
      return null;
    }

    return await response.json();
  } catch (error) {
    console.error('Auth validation failed:', error);
    return null;
  }
}

async function validateRouteAccess(request: NextRequest, route: string) {
  const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
  
  // Get bearer token from HTTP-only cookie
  const token = request.cookies.get(AUTH_COOKIE_NAME)?.value;
  
  if (!token) {
    return { allowed: false };
  }
  
  try {
    const response = await fetch(`${backendUrl}/api/v1/auth/validate-access`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        route,
        method: 'GET',
        app_type: 'frontend',
      }),
    });

    if (!response.ok) {
      return { allowed: false };
    }

    return await response.json();
  } catch (error) {
    console.error('Route validation failed:', error);
    return { allowed: false };
  }
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Public routes that don't require authentication
  const publicRoutes = ['/', '/login', '/register', '/forgot-password', '/reset-password', '/verify-email', '/access-denied', '/unauthorized'];
  const isPublicRoute = publicRoutes.includes(pathname);
  
  // Skip auth for public routes, API routes, and static assets
  if (isPublicRoute || pathname.startsWith('/api') || pathname.startsWith('/_next') || pathname.startsWith('/public')) {
    return NextResponse.next();
  }

  // Protected routes that require authentication
  const protectedRoutes = ['/dashboard', '/analytics', '/my-data', '/trading', '/settings', '/payment'];
  const isProtectedRoute = protectedRoutes.some(route => pathname.startsWith(route));
  
  if (isProtectedRoute) {
    // Validate session with backend
    const authData = await validateWithBackend(request);
    
    if (!authData) {
      // No valid session, redirect to login
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('redirect', pathname);
      return NextResponse.redirect(loginUrl);
    }

    // For premium routes, check subscription tier
    const premiumRoutes = ['/analytics', '/trading'];
    const isPremiumRoute = premiumRoutes.some(route => pathname.startsWith(route));
    
    if (isPremiumRoute) {
      const isPremium = ['premium', 'enterprise', 'platinum', 'gold'].includes(authData.subscription_tier?.toLowerCase());
      if (!isPremium) {
        const upgradeUrl = new URL('/payment', request.url);
        return NextResponse.redirect(upgradeUrl);
      }
    }

    // For admin routes, check admin role
    if (pathname.startsWith('/admin')) {
      const isAdmin = authData.role === 'admin' || authData.role === 'system_administrator';
      if (!isAdmin) {
        const unauthorizedUrl = new URL('/unauthorized', request.url);
        return NextResponse.redirect(unauthorizedUrl);
      }
    }

    // Validate specific route access with backend
    const routeAccess = await validateRouteAccess(request, pathname);
    if (!routeAccess.allowed) {
      // Check if user needs to upgrade or is missing permissions
      const isPremium = ['premium', 'enterprise', 'platinum', 'gold'].includes(authData.subscription_tier?.toLowerCase());
      
      if (!isPremium && premiumRoutes.some(route => pathname.startsWith(route))) {
        const upgradeUrl = new URL('/payment', request.url);
        return NextResponse.redirect(upgradeUrl);
      } else {
        const accessDeniedUrl = new URL('/access-denied', request.url);
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