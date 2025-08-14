/**
 * Enhanced JWT Middleware for Admin Frontend
 * Uses JWT cookie verification with admin module checking and security headers
 */
import { NextRequest, NextResponse } from 'next/server';
import { verifyJWT } from '@epsx/auth-shared';

// Public routes that don't require authentication
const publicRoutes = [
  '/login',
  '/api/auth/callback/epsx-backend',
  '/api/auth/login',
  '/api/auth/signin',
  '/api/auth/logout', 
  '/api/auth/session',
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico'
]

// Routes that require specific admin modules
const adminModuleRoutes: Record<string, string> = {
  '/users': 'user_management',
  '/analytics': 'analytics_specialist', 
  '/billing': 'billing_admin',
  '/settings': 'system_admin',
  '/permissions': 'permission_admin',
  '/permission-profiles': 'user_operations',
  '/stock-ranking-packages': 'package_coordinator'
}

// Admin role hierarchy
const adminRoleHierarchy = {
  moderator: 1,
  admin: 2,
  super_admin: 3,
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Create response with security headers
  const response = NextResponse.next();
  
  // Add security headers for admin app
  response.headers.set('x-pathname', pathname);
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-Robots-Tag', 'noindex, nofollow'); // Admin should not be indexed
  
  // Allow access to public routes
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route)
  );
  
  if (isPublicRoute) {
    return response;
  }
  
  try {
    // Get JWT token from admin-specific httpOnly cookie
    const jwtToken = request.cookies.get('epsx_admin_jwt')?.value;
    
    if (!jwtToken) {
      console.log('🔓 Admin middleware: No JWT token found, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Verify JWT token
    const payload = await verifyJWT(jwtToken);
    
    if (!payload) {
      console.log('🔓 Admin middleware: Invalid JWT token, redirecting to login');
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
      return NextResponse.redirect(loginUrl);
    }
    
    // Ensure user has at least moderator role for admin access
    const userRoleLevel = adminRoleHierarchy[payload.role as keyof typeof adminRoleHierarchy] || 0;
    if (userRoleLevel < 1) {
      console.log(`🚫 Admin middleware: User ${payload.email} has insufficient role (${payload.role}) for admin access`);
      const unauthorizedUrl = new URL('/unauthorized', request.url);
      unauthorizedUrl.searchParams.set('reason', 'insufficient_role');
      return NextResponse.redirect(unauthorizedUrl);
    }
    
    // Check for admin module requirements
    const requiredModule = Object.entries(adminModuleRoutes).find(([route]) => 
      pathname.startsWith(route)
    )?.[1];
    
    if (requiredModule) {
      // Check if user has required admin module or is system admin
      const hasModule = payload.admin_modules?.includes(requiredModule) || 
                       payload.admin_modules?.includes('system_admin') ||
                       payload.role === 'super_admin'; // Super admin has access to everything
      
      if (!hasModule) {
        console.log(`🚫 Admin middleware: User ${payload.email} lacks required module ${requiredModule} for ${pathname}`);
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('module', requiredModule);
        accessDeniedUrl.searchParams.set('route', pathname);
        return NextResponse.redirect(accessDeniedUrl);
      }
    }
    
    // Add admin user info to headers for server components (non-sensitive data only)
    response.headers.set('x-user-id', payload.sub);
    response.headers.set('x-user-role', payload.role);
    response.headers.set('x-user-admin-modules', JSON.stringify(payload.admin_modules || []));
    
    console.log(`🔐 Admin middleware: Authenticated admin ${payload.email} (${payload.role}) accessing ${pathname}`);
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin middleware JWT verification failed:', error);
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('callbackUrl', pathname + request.nextUrl.search);
    return NextResponse.redirect(loginUrl);
  }
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api/public (public API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api/public|_next/static|_next/image|favicon.ico).*)',
  ],
}