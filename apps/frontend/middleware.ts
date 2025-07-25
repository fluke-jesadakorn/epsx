import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { createSecurityMiddleware } from './lib/middleware/security';
import { COOKIE_NAMES } from './lib/cookies';
import { logger } from './lib/logger';

// Enhanced route definitions with permission profile integration
const routePermissions: Record<string, { permission: string; profile?: string; fallbackRole?: string }> = {
  '/dashboard': { permission: 'route:/dashboard', fallbackRole: 'user' },
  '/analytics': { permission: 'route:/analytics/*', profile: 'Silver User', fallbackRole: 'premium' },
  '/my-data': { permission: 'route:/profile/*', fallbackRole: 'user' },
  '/admin': { permission: 'route:/admin/*', fallbackRole: 'admin' },
  '/users': { permission: 'admin.users.view', fallbackRole: 'admin' },
  '/settings': { permission: 'route:/settings', fallbackRole: 'user' },
  '/reports': { permission: 'route:/reports/*', profile: 'Gold User', fallbackRole: 'premium' },
  '/trading': { permission: 'route:/trading/*', profile: 'Silver User', fallbackRole: 'premium' },
  '/payment': { permission: 'route:/payment/*', fallbackRole: 'user' },
  '/premium': { permission: 'route:/premium/*', profile: 'Silver User', fallbackRole: 'premium' },
  '/rankings': { permission: 'api:rankings:read', profile: 'Bronze User', fallbackRole: 'user' },
  '/stock-rankings': { permission: 'api:stock-rankings:read', profile: 'Bronze User', fallbackRole: 'user' },
};

const authRoutes = ['/login', '/register', '/signup'];
const publicRoutes = ['/', '/pricing', '/features', '/contact', '/privacy', '/terms', '/rankings', '/verify-email', '/access-denied', '/unauthorized'];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // Skip middleware for static assets and Next.js internals
  if (
    pathname.startsWith('/_next/') ||
    pathname.startsWith('/favicon.ico') ||
    pathname.includes('.') ||
    pathname.startsWith('/public/')
  ) {
    return NextResponse.next();
  }

  // Apply security middleware first
  const securityMiddleware = createSecurityMiddleware();
  const securityResponse = securityMiddleware(request);
  
  // If security middleware blocks the request, return that response
  if (securityResponse.status !== 200) {
    return securityResponse;
  }
  
  // Check if route is public (no authentication required)
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route + '/')
  );
  
  const isAuthRoute = authRoutes.some(route => pathname.startsWith(route));
  const isProtectedRoute = Object.keys(routePermissions).some(route => 
    pathname.startsWith(route)
  );
  
  // Get the session token from cookies
  const token = request.cookies.get(COOKIE_NAMES.SESSION)?.value;
  
  // Debug logging (only in development)
  if (process.env.NODE_ENV === 'development') {
    logger.debug('Middleware route check', { 
      pathname, 
      isPublicRoute, 
      isProtectedRoute, 
      isAuthRoute,
      hasToken: !!token,
      tokenPreview: token ? token.substring(0, 10) + '...' : 'none'
    });
  }
  
  // Allow public routes without authentication
  if (isPublicRoute) {
    const response = NextResponse.next();
    return addSecurityHeaders(response);
  }
  
  // Redirect to login if accessing protected route without token
  if (isProtectedRoute && !token) {
    if (process.env.NODE_ENV === 'development') {
      logger.debug('Redirecting to login - no token for protected route', { pathname });
    }
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }
  
  // Redirect to dashboard if accessing auth routes with valid token
  if (isAuthRoute && token) {
    // Only redirect if the token is actually valid - check with backend first
    try {
      const hasAccess = await checkRoutePermissions(token, '/dashboard');
      if (hasAccess.allowed) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
      }
      // If token is invalid, clear it and allow login page to load
    } catch (error) {
      // If check fails, allow login page to load (don't redirect)
    }
  }
  
  // For protected routes with token, check permissions
  if (isProtectedRoute && token) {
    try {
      const hasAccess = await checkPermissionProfileAccess(token, pathname);
      
      if (!hasAccess.allowed) {
        if (process.env.NODE_ENV === 'development') {
          logger.warn('Access denied for route', { 
            pathname, 
            reason: hasAccess.reason,
            requiredPermission: hasAccess.requiredPermission,
            userPermissions: hasAccess.userPermissions?.slice(0, 5) // Log first 5 for debugging
          });
        }
        
        // Redirect to access denied page with detailed info
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('route', pathname);
        accessDeniedUrl.searchParams.set('reason', hasAccess.reason);
        if (hasAccess.requiredPermission) {
          accessDeniedUrl.searchParams.set('permission', hasAccess.requiredPermission);
        }
        return NextResponse.redirect(accessDeniedUrl);
      }
    } catch (error) {
      logger.error('Error checking route permissions', { error: error instanceof Error ? error.message : error, pathname });
      // Continue with default behavior on error
    }
  }
  
  const response = NextResponse.next();
  return addSecurityHeaders(response);
}

/**
 * Add security headers to all responses
 */
function addSecurityHeaders(response: NextResponse): NextResponse {
  // Security headers
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('Referrer-Policy', 'origin-when-cross-origin');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  
  // Content Security Policy
  const csp = [
    "default-src 'self'",
    "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://www.googletagmanager.com https://cdnjs.cloudflare.com",
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com",
    "img-src 'self' data: https: blob:",
    "connect-src 'self' https: wss:",
    "frame-src 'none'",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
  ].join('; ');
  
  response.headers.set('Content-Security-Policy', csp);
  
  // Additional security headers
  response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
  response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
  
  return response;
}


// Permission cache for performance optimization
interface PermissionCacheEntry {
  permissions: string[];
  role: string;
  profiles: string[];
  timestamp: number;
  ttl: number; // 5 minutes
}

const permissionCache = new Map<string, PermissionCacheEntry>();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

/**
 * Enhanced permission checking with permission profile integration
 */
async function checkPermissionProfileAccess(
  token: string, 
  route: string
): Promise<{ 
  allowed: boolean; 
  requiredPermission?: string; 
  reason: string;
  userPermissions?: string[];
  userRole?: string;
  userProfiles?: string[];
}> {
  try {
    // Check cache first
    const cacheKey = `perms_${token.slice(-10)}`; // Use last 10 chars as key
    const cached = permissionCache.get(cacheKey);
    
    let userPermissions: string[];
    let userRole: string;
    let userProfiles: string[];
    
    if (cached && (Date.now() - cached.timestamp) < CACHE_TTL) {
      userPermissions = cached.permissions;
      userRole = cached.role;
      userProfiles = cached.profiles;
    } else {
      // Fetch from backend
      const baseUrl = process.env.NEXTAUTH_URL || process.env.BACKEND_URL;
      if (!baseUrl) {
        throw new Error('Backend URL environment variable is required');
      }
      const response = await fetch(`${baseUrl}/api/v1/auth/profile`, {
        method: 'GET',
        headers: {
          'Cookie': `sess_id=${token}`,
          'User-Agent': 'Next.js Middleware',
        },
      });

      if (!response.ok) {
        return { allowed: false, reason: 'User authentication failed' };
      }

      const userData = await response.json();
      userPermissions = userData.permissions || [];
      userRole = userData.role || 'user';
      userProfiles = userData.permission_profiles || [];
      
      // Cache the permissions
      permissionCache.set(cacheKey, {
        permissions: userPermissions,
        role: userRole,
        profiles: userProfiles,
        timestamp: Date.now(),
        ttl: CACHE_TTL
      });
      
      // Clean expired entries (simple cleanup)
      if (permissionCache.size > 100) {
        const now = Date.now();
        for (const [key, entry] of permissionCache.entries()) {
          if (now - entry.timestamp > CACHE_TTL) {
            permissionCache.delete(key);
          }
        }
      }
    }
    
    // Find matching route configuration
    const routeConfig = Object.keys(routePermissions).find(routePattern => {
      if (routePattern.endsWith('/*')) {
        const prefix = routePattern.slice(0, -2);
        return route.startsWith(prefix);
      }
      return route === routePattern || route.startsWith(routePattern + '/');
    });

    if (!routeConfig) {
      return { 
        allowed: true, 
        reason: 'No permission required for this route',
        userPermissions,
        userRole,
        userProfiles
      };
    }

    const config = routePermissions[routeConfig];
    const requiredPermission = config.permission;

    // Check permission profile access
    if (config.profile && userProfiles.includes(config.profile)) {
      return { 
        allowed: true, 
        reason: `Access granted via permission profile: ${config.profile}`,
        requiredPermission,
        userPermissions,
        userRole,
        userProfiles
      };
    }

    // Check specific permission
    if (userPermissions.includes(requiredPermission)) {
      return { 
        allowed: true, 
        reason: 'Access granted via specific permission',
        requiredPermission,
        userPermissions,
        userRole,
        userProfiles
      };
    }
    
    // Check for wildcard permissions (e.g., "admin.*" covers "admin.users.view")
    const hasWildcardPermission = userPermissions.some((permission: string) => {
      if (permission.endsWith('.*') || permission.endsWith(':*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.') || requiredPermission.startsWith(prefix + ':');
      }
      if (permission === '*') {
        return true;
      }
      return false;
    });

    if (hasWildcardPermission) {
      return { 
        allowed: true, 
        reason: 'Access granted via wildcard permission',
        requiredPermission,
        userPermissions,
        userRole,
        userProfiles
      };
    }

    // Fallback to role-based access
    if (config.fallbackRole && checkRoleBasedAccess(userRole, config.fallbackRole)) {
      return { 
        allowed: true, 
        reason: `Access granted via role-based fallback: ${userRole}`,
        requiredPermission,
        userPermissions,
        userRole,
        userProfiles
      };
    }

    return {
      allowed: false,
      requiredPermission,
      reason: `Access denied: Missing permission '${requiredPermission}' or profile '${config.profile || 'N/A'}' or role '${config.fallbackRole || 'N/A'}'`,
      userPermissions,
      userRole,
      userProfiles
    };
  } catch (error) {
    logger.error('Error in checkPermissionProfileAccess', { error: error instanceof Error ? error.message : error, route });
    // Allow access on error to prevent blocking users due to technical issues
    return { allowed: true, reason: 'Permission check failed, allowing access' };
  }
}

/**
 * Check role-based access for fallback scenarios
 */
function checkRoleBasedAccess(userRole: string, requiredRole: string): boolean {
  const roleHierarchy: Record<string, number> = {
    'user': 1,
    'premium': 2,
    'moderator': 3,
    'admin': 4,
    'super_admin': 5
  };

  const userLevel = roleHierarchy[userRole.toLowerCase()] || 0;
  const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1;

  return userLevel >= requiredLevel;
}

/**
 * Legacy function for backward compatibility
 */
async function checkRoutePermissions(
  token: string, 
  route: string
): Promise<{ allowed: boolean; requiredPermission?: string; reason: string }> {
  const result = await checkPermissionProfileAccess(token, route);
  return {
    allowed: result.allowed,
    requiredPermission: result.requiredPermission,
    reason: result.reason
  };
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
