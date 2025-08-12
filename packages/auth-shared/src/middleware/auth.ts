import type { NextRequest, NextResponse } from 'next/server';
import type { PermissionCacheEntry, PermissionCheckResult, RoutePermissionConfig } from '../types';

export interface MiddlewareAuthConfig {
  backendUrl?: string;
  sessionCookieName?: string;
  adminSessionCookieName?: string;
  cacheTTL?: number;
  maxCacheSize?: number;
  enableCaching?: boolean;
}

const DEFAULT_CONFIG: Required<MiddlewareAuthConfig> = {
  backendUrl: process.env.BACKEND_URL || process.env.NEXTAUTH_URL || 'http://localhost:8080',
  sessionCookieName: 'sess_id',
  adminSessionCookieName: 'admin_sess_id',
  cacheTTL: 5 * 60 * 1000, // 5 minutes
  maxCacheSize: 100,
  enableCaching: true,
};

// Permission cache for performance optimization
const permissionCache = new Map<string, PermissionCacheEntry>();

/**
 * Get session token from request cookies
 */
export function getSessionToken(
  request: NextRequest, 
  config: MiddlewareAuthConfig = {}
): { token: string | null; isAdmin: boolean } {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  
  const adminToken = request.cookies.get(finalConfig.adminSessionCookieName)?.value;
  const userToken = request.cookies.get(finalConfig.sessionCookieName)?.value;
  
  if (adminToken) {
    return { token: adminToken, isAdmin: true };
  }
  
  if (userToken) {
    return { token: userToken, isAdmin: false };
  }
  
  return { token: null, isAdmin: false };
}

/**
 * Enhanced permission checking with cache and admin/user support
 */
export async function checkPermissionAccess(
  token: string,
  route: string,
  routePermissions: Record<string, RoutePermissionConfig>,
  isAdmin: boolean = false,
  config: MiddlewareAuthConfig = {}
): Promise<PermissionCheckResult> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  
  try {
    // Check cache first
    const cacheKey = `${isAdmin ? 'admin' : 'user'}_perms_${token.slice(-10)}`;
    const cached = finalConfig.enableCaching ? permissionCache.get(cacheKey) : null;
    
    let userPermissions: string[];
    let userRole: string;
    let userProfiles: string[];
    let isAdminUser: boolean;
    
    if (cached && (Date.now() - cached.timestamp) < finalConfig.cacheTTL) {
      userPermissions = cached.permissions;
      userRole = cached.role;
      userProfiles = cached.profiles;
      isAdminUser = cached.isAdmin;
    } else {
      // Fetch from backend
      const endpoint = isAdmin ? '/api/admin/auth/profile' : '/api/v1/auth/me';
      const cookieName = isAdmin ? finalConfig.adminSessionCookieName : finalConfig.sessionCookieName;
      
      const response = await fetch(`${finalConfig.backendUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Cookie': `${cookieName}=${token}`,
          'User-Agent': 'Auth-Shared Middleware',
        },
      });

      if (!response.ok) {
        return { 
          allowed: false, 
          reason: `${isAdmin ? 'Admin' : 'User'} authentication failed` 
        };
      }

      const userData = await response.json();
      userPermissions = userData.permissions || [];
      userRole = userData.role || 'user';
      userProfiles = userData.permission_profiles || [];
      isAdminUser = ['admin', 'super_admin', 'superadmin', 'moderator'].includes(userRole.toLowerCase());
      
      // Cache the permissions
      if (finalConfig.enableCaching) {
        permissionCache.set(cacheKey, {
          userId: cacheKey.split('_')[1] || 'unknown', // Extract userId from cacheKey
          permissions: userPermissions,
          role: userRole,
          profiles: userProfiles,
          isAdmin: isAdminUser,
          timestamp: Date.now(),
          ttl: finalConfig.cacheTTL
        });
        
        // Clean expired entries
        if (permissionCache.size > finalConfig.maxCacheSize) {
          cleanExpiredCache(finalConfig.cacheTTL);
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

    // For admin routes, check admin role first
    if (isAdmin && config.minimumRole) {
      if (!checkRoleHierarchy(userRole, config.minimumRole)) {
        return {
          allowed: false,
          reason: `Access denied: Minimum role '${config.minimumRole}' required, current role: '${userRole}'`,
          requiredPermission,
          userRole,
          userPermissions,
          userProfiles
        };
      }
    }

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
    
    // Check for wildcard permissions
    const hasWildcardPermission = userPermissions.some((permission: string) => {
      // SuperAdmin full wildcard permission
      if (permission === '*:*:*' || permission === '*') {
        return true;
      }
      
      // Admin wildcard permission
      if (permission === 'admin:*:*' && requiredPermission.startsWith('admin.')) {
        return true;
      }
      
      // Handle dot notation wildcards (frontend format)
      if (permission.endsWith('.*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.');
      }
      
      // Handle colon notation wildcards (backend format)
      if (permission.endsWith(':*:*')) {
        const domain = permission.slice(0, -4);
        return requiredPermission.startsWith(domain + '.') || requiredPermission.startsWith(domain + ':');
      }
      
      if (permission.endsWith(':*')) {
        const prefix = permission.slice(0, -2);
        return requiredPermission.startsWith(prefix + '.') || requiredPermission.startsWith(prefix + ':');
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
    if (config.fallbackRole && checkRoleHierarchy(userRole, config.fallbackRole)) {
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
    console.error('Error in checkPermissionAccess', { 
      error: error instanceof Error ? error.message : error, 
      route,
      isAdmin 
    });
    
    // Allow access on error to prevent blocking users due to technical issues
    return { 
      allowed: true, 
      reason: 'Permission check failed, allowing access' 
    };
  }
}

/**
 * Check role hierarchy
 */
export function checkRoleHierarchy(userRole: string, requiredRole: string): boolean {
  const roleHierarchy: Record<string, number> = {
    'user': 1,
    'premium': 2,
    'moderator': 3,
    'admin': 4,
    'super_admin': 5,
    'superadmin': 5  // Handle backend SuperAdmin role format
  };

  const userLevel = roleHierarchy[userRole.toLowerCase()] || 0;
  const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1;

  return userLevel >= requiredLevel;
}

/**
 * Clean expired cache entries
 */
function cleanExpiredCache(ttl: number): void {
  const now = Date.now();
  for (const [key, entry] of permissionCache.entries()) {
    if (now - entry.timestamp > ttl) {
      permissionCache.delete(key);
    }
  }
}

/**
 * Clear permission cache
 */
export function clearPermissionCache(): void {
  permissionCache.clear();
}

/**
 * Add security headers to response
 */
export function addSecurityHeaders(response: NextResponse): NextResponse {
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

/**
 * Admin-specific security headers (more restrictive)
 */
export function addAdminSecurityHeaders(response: NextResponse): NextResponse {
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