import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// Admin route definitions with strict permission requirements
const adminRoutePermissions: Record<string, { 
  permission: string; 
  profile?: string; 
  minimumRole: string;
  description: string;
}> = {
  '/dashboard': { 
    permission: 'admin.dashboard.view', 
    minimumRole: 'admin',
    description: 'Admin dashboard access'
  },
  '/users': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    minimumRole: 'admin',
    description: 'User management interface'
  },
  '/user-management': { 
    permission: 'admin.users.manage', 
    profile: 'Admin Assistant',
    minimumRole: 'admin',
    description: 'Enhanced user management'
  },
  '/permission-profiles': {
    permission: 'admin.permission_profiles.manage',
    profile: 'System Administrator',
    minimumRole: 'admin',
    description: 'Permission profile management'
  },
  '/stock-ranking-packages': {
    permission: 'admin.stock_rankings.manage',
    profile: 'Content Manager',
    minimumRole: 'admin', 
    description: 'Stock ranking package management'
  },
  '/analytics': { 
    permission: 'admin.analytics.view', 
    profile: 'Admin Assistant',
    minimumRole: 'admin',
    description: 'Admin analytics dashboard'
  },
  '/system': { 
    permission: 'admin.system.configure', 
    profile: 'System Administrator',
    minimumRole: 'super_admin',
    description: 'System configuration'
  },
  '/audit': { 
    permission: 'admin.audit.view', 
    profile: 'Admin Assistant',
    minimumRole: 'admin',
    description: 'Audit log access'
  },
  '/api/admin': { 
    permission: 'api:admin:*', 
    minimumRole: 'admin',
    description: 'Admin API access'
  }
};

const authRoutes = ['/login'];
const publicRoutes = ['/login', '/access-denied', '/unauthorized'];

// Permission cache for performance
interface AdminPermissionCacheEntry {
  permissions: string[];
  role: string;
  profiles: string[];
  isAdmin: boolean;
  timestamp: number;
}

const adminPermissionCache = new Map<string, AdminPermissionCacheEntry>();
const ADMIN_CACHE_TTL = 3 * 60 * 1000; // 3 minutes for admin routes

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
  
  // Get admin session token
  const token = request.cookies.get('admin_sess_id')?.value || 
                request.cookies.get('sess_id')?.value;
  
  // Debug logging for admin routes
  if (process.env.NODE_ENV === 'development') {
    console.log('[Admin Middleware]', { 
      pathname, 
      isPublicRoute, 
      isProtectedRoute, 
      isAuthRoute,
      hasToken: !!token 
    });
  }
  
  // Allow public routes
  if (isPublicRoute) {
    return addSecurityHeaders(NextResponse.next());
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
      const hasAdminAccess = await checkAdminPermissions(token, '/dashboard');
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
      const hasAccess = await checkAdminPermissions(token, pathname);
      
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
  
  return addSecurityHeaders(NextResponse.next());
}

/**
 * Check admin permissions with enhanced security
 */
async function checkAdminPermissions(
  token: string, 
  route: string
): Promise<{ 
  allowed: boolean; 
  reason: string;
  requiredPermission?: string;
  userRole?: string;
  userPermissions?: string[];
}> {
  try {
    // Check cache first
    const cacheKey = `admin_perms_${token.slice(-10)}`;
    const cached = adminPermissionCache.get(cacheKey);
    
    let userPermissions: string[];
    let userRole: string;
    let userProfiles: string[];
    let isAdmin: boolean;
    
    if (cached && (Date.now() - cached.timestamp) < ADMIN_CACHE_TTL) {
      userPermissions = cached.permissions;
      userRole = cached.role;
      userProfiles = cached.profiles;
      isAdmin = cached.isAdmin;
    } else {
      // Fetch admin user data from backend
      const baseUrl = process.env.NEXTAUTH_URL || process.env.BACKEND_URL;
      if (!baseUrl) {
        throw new Error('Backend URL environment variable is required for admin');
      }
      
      const response = await fetch(`${baseUrl}/api/admin/auth/profile`, {
        method: 'GET',
        headers: {
          'Cookie': `sess_id=${token}`,
          'User-Agent': 'Admin Next.js Middleware',
        },
      });

      if (!response.ok) {
        return { 
          allowed: false, 
          reason: 'Admin authentication failed or insufficient privileges' 
        };
      }

      const userData = await response.json();
      userPermissions = userData.permissions || [];
      userRole = userData.role || '';
      userProfiles = userData.permission_profiles || [];
      isAdmin = ['admin', 'super_admin', 'moderator'].includes(userRole.toLowerCase());
      
      // Only cache if user has admin privileges
      if (isAdmin) {
        adminPermissionCache.set(cacheKey, {
          permissions: userPermissions,
          role: userRole,
          profiles: userProfiles,
          isAdmin,
          timestamp: Date.now()
        });
      }
    }
    
    // First check: must be admin role
    if (!isAdmin) {
      return {
        allowed: false,
        reason: `Access denied: Admin role required, current role: ${userRole}`,
        userRole
      };
    }
    
    // Find matching admin route configuration
    const routeConfig = Object.keys(adminRoutePermissions).find(routePattern => {
      return route === routePattern || route.startsWith(routePattern + '/');
    });

    if (!routeConfig) {
      // For unmapped admin routes, require basic admin access
      return { 
        allowed: isAdmin, 
        reason: isAdmin ? 'Basic admin access granted' : 'Admin role required',
        userRole
      };
    }

    const config = adminRoutePermissions[routeConfig];
    const requiredPermission = config.permission;

    // Check minimum role requirement
    if (!checkAdminRoleHierarchy(userRole, config.minimumRole)) {
      return {
        allowed: false,
        reason: `Access denied: Minimum role '${config.minimumRole}' required, current role: '${userRole}'`,
        requiredPermission,
        userRole
      };
    }

    // Check permission profile requirement
    if (config.profile && !userProfiles.includes(config.profile)) {
      return {
        allowed: false,
        reason: `Access denied: Required permission profile '${config.profile}' not assigned`,
        requiredPermission,
        userRole,
        userPermissions
      };
    }

    // Check specific permission
    if (userPermissions.includes(requiredPermission)) {
      return { 
        allowed: true, 
        reason: `Admin access granted via specific permission: ${requiredPermission}`,
        requiredPermission,
        userRole,
        userPermissions
      };
    }
    
    // Check wildcard admin permissions
    const hasWildcardPermission = userPermissions.some((permission: string) => {
      if (permission === 'admin:*' || permission === 'admin.*') {
        return true;
      }
      if (permission.endsWith(':*') || permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + ':') || requiredPermission.startsWith(prefix + '.');
      }
      if (permission === '*') {
        return true;
      }
      return false;
    });

    if (hasWildcardPermission) {
      return { 
        allowed: true, 
        reason: 'Admin access granted via wildcard permission',
        requiredPermission,
        userRole,
        userPermissions
      };
    }

    return {
      allowed: false,
      reason: `Access denied: Missing required permission '${requiredPermission}' for admin route`,
      requiredPermission,
      userRole,
      userPermissions
    };
  } catch (error) {
    console.error('[Admin Permission Check Error]', { error: error instanceof Error ? error.message : error, route });
    // For admin routes, deny access on error for security
    return { 
      allowed: false, 
      reason: 'Permission check failed - access denied for security' 
    };
  }
}

/**
 * Check admin role hierarchy
 */
function checkAdminRoleHierarchy(userRole: string, requiredRole: string): boolean {
  const adminRoleHierarchy: Record<string, number> = {
    'moderator': 2,
    'admin': 3,
    'super_admin': 4
  };

  const userLevel = adminRoleHierarchy[userRole.toLowerCase()] || 0;
  const requiredLevel = adminRoleHierarchy[requiredRole.toLowerCase()] || 1;

  return userLevel >= requiredLevel;
}

/**
 * Add enhanced security headers for admin interface
 */
function addSecurityHeaders(response: NextResponse): NextResponse {
  // Enhanced security headers for admin
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('Referrer-Policy', 'no-referrer');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  
  // Strict CSP for admin interface
  const adminCsp = [
    "default-src 'self'",
    "script-src 'self' 'unsafe-inline' 'unsafe-eval'", // Allow inline and eval for admin dashboards
    "style-src 'self' 'unsafe-inline'",
    "font-src 'self'",
    "img-src 'self' data: https:",
    "connect-src 'self'",
    "frame-src 'none'",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
  ].join('; ');
  
  response.headers.set('Content-Security-Policy', adminCsp);
  
  // Additional admin security headers
  response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=(), usb=(), bluetooth=()');
  response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains; preload');
  response.headers.set('X-Admin-Interface', 'true');
  
  return response;
}

export const config = {
  matcher: [
    /*
     * Match all admin routes except static files
     */
    '/((?!api|_next/static|_next/image|favicon.ico|public).*)',
  ],
};