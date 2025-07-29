import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { 
  getSessionToken, 
  checkPermissionAccess, 
  addAdminSecurityHeaders 
} from '@epsx/auth-shared/middleware';
import type { RoutePermissionConfig } from '@epsx/auth-shared';

// Admin route definitions with strict permission requirements
const adminRoutePermissions: Record<string, RoutePermissionConfig> = {
  '/': { 
    permission: 'admin.dashboard.view', 
    fallbackRole: 'admin',
    description: 'Admin dashboard access'
  },
  '/dashboard': { 
    permission: 'admin.dashboard.view', 
    fallbackRole: 'admin',
    description: 'Admin dashboard access'
  },
  '/users': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'User management interface'
  },
  '/user-management': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Enhanced user management'
  },
  '/permission-profiles': {
    permission: 'admin.permission_profiles.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Permission profile management'
  },
  '/stock-ranking-packages': {
    permission: 'admin.stock_rankings.manage',
    profile: 'Content Manager',
    fallbackRole: 'admin', 
    description: 'Stock ranking package management'
  },
  '/analytics': { 
    permission: 'admin.analytics.view', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Admin analytics dashboard'
  },
  '/iam': {
    permission: 'admin.iam.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Identity and Access Management'
  },
  '/settings': {
    permission: 'admin.settings.manage',
    profile: 'System Administrator', 
    fallbackRole: 'admin',
    description: 'Admin settings management'
  },
  '/database': {
    permission: 'admin.database.manage',
    profile: 'System Administrator',
    fallbackRole: 'admin',
    description: 'Database management interface'
  },
  '/system': { 
    permission: 'admin.system.configure', 
    profile: 'System Administrator',
    fallbackRole: 'super_admin',
    description: 'System configuration'
  },
  '/audit': { 
    permission: 'admin.audit.view', 
    profile: 'Admin Assistant',
    fallbackRole: 'admin',
    description: 'Audit log access'
  },
  '/api/admin': { 
    permission: 'api:admin:*', 
    fallbackRole: 'admin',
    description: 'Admin API access'
  }
};

const authRoutes = ['/login'];
const publicRoutes = ['/login', '/access-denied', '/unauthorized'];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Skip middleware for static assets
  if (
    pathname.startsWith('/_next/') ||
    pathname.startsWith('/favicon.ico') ||
    pathname.includes('.') ||
    pathname.startsWith('/public/')
  ) {
    return NextResponse.next();
  }

  // Check if route is public
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  const isAuthRoute = authRoutes.some(route => pathname.startsWith(route));
  const isProtectedRoute = Object.keys(adminRoutePermissions).some(route => 
    pathname.startsWith(route)
  );
  
  // Get session token using shared utility
  const { token, isAdmin } = getSessionToken(request, {
    adminSessionCookieName: 'admin_sess_id',
    sessionCookieName: 'sess_id'
  });
  
  // Debug logging for admin routes
  if (process.env.NODE_ENV === 'development') {
    console.log('[Admin Middleware]', { 
      pathname, 
      isPublicRoute, 
      isProtectedRoute, 
      isAuthRoute,
      hasToken: !!token,
      isAdmin
    });
  }
  
  // Allow public routes
  if (isPublicRoute) {
    return addAdminSecurityHeaders(NextResponse.next());
  }
  
  // Redirect to admin login if no token
  if (!token) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }
  
  // Redirect authenticated admin users away from login
  if (isAuthRoute && token) {
    try {
      const hasAdminAccess = await checkPermissionAccess(
        token, 
        '/dashboard', 
        adminRoutePermissions,
        isAdmin,
        {
          adminSessionCookieName: 'admin_sess_id',
          sessionCookieName: 'sess_id'
        }
      );
      if (hasAdminAccess.allowed) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
      }
    } catch (error) {
      // Allow login page to load on error
    }
  }
  
  // For protected admin routes, enforce strict permission checking
  if (isProtectedRoute && token) {
    try {
      const hasAccess = await checkPermissionAccess(
        token,
        pathname,
        adminRoutePermissions,
        isAdmin,
        {
          adminSessionCookieName: 'admin_sess_id',
          sessionCookieName: 'sess_id',
          cacheTTL: 3 * 60 * 1000 // 3 minutes for admin routes
        }
      );
      
      if (!hasAccess.allowed) {
        console.warn('[Admin Access Denied]', { 
          pathname, 
          reason: hasAccess.reason,
          userRole: hasAccess.userRole,
          requiredPermission: hasAccess.requiredPermission
        });
        
        // Redirect to access denied with admin context
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('route', pathname);
        accessDeniedUrl.searchParams.set('reason', hasAccess.reason);
        accessDeniedUrl.searchParams.set('context', 'admin');
        if (hasAccess.requiredPermission) {
          accessDeniedUrl.searchParams.set('permission', hasAccess.requiredPermission);
        }
        return NextResponse.redirect(accessDeniedUrl);
      }
      
      // Log successful admin access
      if (process.env.NODE_ENV === 'development') {
        console.log('[Admin Access Granted]', {
          pathname,
          reason: hasAccess.reason,
          userRole: hasAccess.userRole
        });
      }
    } catch (error) {
      console.error('[Admin Middleware Error]', { error: error instanceof Error ? error.message : error, pathname });
      // For admin routes, be more restrictive on errors
      const errorUrl = new URL('/access-denied', request.url);
      errorUrl.searchParams.set('route', pathname);
      errorUrl.searchParams.set('reason', 'System error during permission check');
      errorUrl.searchParams.set('context', 'admin');
      return NextResponse.redirect(errorUrl);
    }
  }
  
  return addAdminSecurityHeaders(NextResponse.next());
}


export const config = {
  matcher: [
    /*
     * Match all admin routes except static files
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};