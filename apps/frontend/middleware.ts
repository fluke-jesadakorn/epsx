import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { createSecurityMiddleware } from './lib/middleware/security';
import { COOKIE_NAMES } from './lib/cookies';

// Route definitions with permission requirements
const routePermissions: Record<string, string> = {
  '/dashboard': 'app.dashboard.view',
  '/analytics': 'analytics.view',
  '/my-data': 'data.own.view',
  '/admin': 'admin.access',
  '/users': 'admin.users.view',
  '/settings': 'settings.view',
  '/reports': 'reports.view',
  '/trading': 'trading.view',
  '/payment': 'payment.access',
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
    console.log('Middleware:', { 
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
      console.log('Redirecting to login - no token for protected route:', pathname);
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
      const hasAccess = await checkRoutePermissions(token, pathname);
      
      if (!hasAccess.allowed) {
        if (process.env.NODE_ENV === 'development') {
          console.log('Access denied for route:', pathname, 'Reason:', hasAccess.reason);
        }
        
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
  timestamp: number;
  ttl: number; // 5 minutes
}

const permissionCache = new Map<string, PermissionCacheEntry>();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

/**
 * Check if user has permission to access a route with caching
 */
async function checkRoutePermissions(
  token: string, 
  route: string
): Promise<{ allowed: boolean; requiredPermission?: string; reason: string }> {
  try {
    // Check cache first
    const cacheKey = `perms_${token.slice(-10)}`; // Use last 10 chars as key
    const cached = permissionCache.get(cacheKey);
    
    let userPermissions: string[];
    
    if (cached && (Date.now() - cached.timestamp) < CACHE_TTL) {
      userPermissions = cached.permissions;
    } else {
      // Fetch from backend
      const response = await fetch(`${process.env.NEXTAUTH_URL || 'http://localhost:3000'}/api/auth/me`, {
        method: 'GET',
        headers: {
          'Cookie': `sess_id=${token}`,
        },
      });

      if (!response.ok) {
        return { allowed: false, reason: 'User authentication failed' };
      }

      const userData = await response.json();
      userPermissions = userData.permissions || [];
      
      // Cache the permissions
      permissionCache.set(cacheKey, {
        permissions: userPermissions,
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
