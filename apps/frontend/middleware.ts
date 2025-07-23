import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// Route definitions with permission requirements
const routePermissions: Record<string, string> = {
  '/dashboard': 'app.dashboard.view',
  '/analytics': 'analytics.view',
  '/my-data': 'data.own.view',
  '/admin': 'admin.access',
  '/users': 'admin.users.view',
  '/settings': 'settings.view',
  '/reports': 'reports.view',
};

const authRoutes = ['/login', '/register'];
const publicRoutes = ['/', '/pricing', '/features', '/contact'];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Check if route is public (no authentication required)
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  const isAuthRoute = authRoutes.some(route => pathname.startsWith(route));
  const isProtectedRoute = Object.keys(routePermissions).some(route => 
    pathname.startsWith(route)
  );
  
  // Get the session token from cookies
  const token = request.cookies.get('__session')?.value;
  
  // Debug logging
  console.log('Middleware:', { 
    pathname, 
    isPublicRoute, 
    isProtectedRoute, 
    isAuthRoute,
    hasToken: !!token 
  });
  
  // Allow public routes without authentication
  if (isPublicRoute) {
    return NextResponse.next();
  }
  
  // Redirect to login if accessing protected route without token
  if (isProtectedRoute && !token) {
    console.log('Redirecting to login - no token for protected route:', pathname);
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }
  
  // Redirect to dashboard if accessing auth routes with token
  if (isAuthRoute && token) {
    return NextResponse.redirect(new URL('/dashboard', request.url));
  }
  
  // For protected routes with token, check permissions
  if (isProtectedRoute && token) {
    try {
      const hasAccess = await checkRoutePermissions(token, pathname);
      
      if (!hasAccess.allowed) {
        console.log('Access denied for route:', pathname, 'Reason:', hasAccess.reason);
        
        // Redirect to access denied page or dashboard
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('route', pathname);
        accessDeniedUrl.searchParams.set('reason', hasAccess.reason);
        return NextResponse.redirect(accessDeniedUrl);
      }
    } catch (error) {
      console.error('Error checking route permissions:', error);
      // Continue with default behavior on error
    }
  }
  
  return NextResponse.next();
}


/**
 * Check if user has permission to access a route using backend API
 */
async function checkRoutePermissions(
  token: string, 
  route: string
): Promise<{ allowed: boolean; requiredPermission?: string; reason: string }> {
  try {
    // Use the existing /api/auth/me route to get user data from backend
    const response = await fetch(`${process.env.NEXTAUTH_URL || 'http://localhost:3000'}/api/auth/me`, {
      method: 'GET',
      headers: {
        'Cookie': `__session=${token}`,
      },
    });

    if (!response.ok) {
      return { allowed: false, reason: 'User authentication failed' };
    }

    const userData = await response.json();
    const userPermissions = userData.permissions || [];
    const requiredPermission = routePermissions[route];

    if (!requiredPermission) {
      return { allowed: true, reason: 'No permission required for this route' };
    }

    // Check if user has the specific permission
    const hasPermission = userPermissions.includes(requiredPermission);
    
    // Check for wildcard permissions (e.g., "admin.*" covers "admin.users.view")
    const hasWildcardPermission = userPermissions.some((permission: string) => {
      if (permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.');
      }
      return false;
    });

    const allowed = hasPermission || hasWildcardPermission || userPermissions.includes('*');

    return {
      allowed,
      requiredPermission,
      reason: allowed ? 'Permission granted' : `Missing required permission: ${requiredPermission}`
    };
  } catch (error) {
    console.error('Error in checkRoutePermissions:', error);
    // Allow access on error to prevent blocking users due to technical issues
    return { allowed: true, reason: 'Permission check failed, allowing access' };
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
     * - public folder
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};
