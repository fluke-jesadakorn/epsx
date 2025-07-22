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
  
  // For protected routes with token, check template-based permissions
  if (isProtectedRoute && token) {
    try {
      // Get user ID from token (simplified - in production, verify JWT)
      const userId = await getUserIdFromToken(token);
      
      if (userId) {
        const hasAccess = await checkRoutePermissions(userId, pathname);
        
        if (!hasAccess.allowed) {
          console.log('Access denied for route:', pathname, 'Reason:', hasAccess.reason);
          
          // Redirect to access denied page or dashboard
          const accessDeniedUrl = new URL('/access-denied', request.url);
          accessDeniedUrl.searchParams.set('route', pathname);
          accessDeniedUrl.searchParams.set('reason', hasAccess.reason);
          return NextResponse.redirect(accessDeniedUrl);
        }
      }
    } catch (error) {
      console.error('Error checking route permissions:', error);
      // Continue with default behavior on error
    }
  }
  
  return NextResponse.next();
}

/**
 * Extract user ID from session token
 * In production, this should verify the JWT and extract claims
 */
async function getUserIdFromToken(token: string): Promise<string | null> {
  try {
    // This is a simplified version - in production, verify JWT signature
    // and extract user ID from claims
    const payload = JSON.parse(atob(token.split('.')[1]));
    return payload.sub || payload.uid || null;
  } catch (error) {
    console.error('Error extracting user ID from token:', error);
    return null;
  }
}

/**
 * Check if user has permission to access a route using template system
 */
async function checkRoutePermissions(
  userId: string, 
  route: string
): Promise<{ allowed: boolean; requiredPermission?: string; reason: string }> {
  try {
    // Dynamic import to avoid edge runtime issues
    const { templateEvaluationService } = await import('@/lib/template-evaluation');
    const { doc, getDoc } = await import('firebase/firestore');
    const { db } = await import('@/lib/firebase-iam');
    const { PackageTier } = await import('@epsx/types');
    
    if (!db) {
      return { allowed: true, reason: 'Firebase not available, allowing access' };
    }
    
    // Get user data
    const userDoc = await getDoc(doc(db, 'users', userId));
    
    if (!userDoc.exists()) {
      return { allowed: false, reason: 'User data not found' };
    }
    
    const userData = userDoc.data();
    
    // Build user context
    const context = {
      userId,
      packageTier: userData.packageTier || PackageTier.FREE,
      staticPermissions: userData.permissions || [],
      roles: userData.roles || [],
    };
    
    // Check route permissions using template system
    return await templateEvaluationService.getRoutePermissions(context, route);
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
