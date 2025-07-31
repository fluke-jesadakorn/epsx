import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { 
  getSessionToken, 
  checkPermissionAccess, 
  addSecurityHeaders, 
  addAdminSecurityHeaders,
  type MiddlewareAuthConfig
} from './auth';
import type { RoutePermissionConfig } from '../types';

export interface UnifiedMiddlewareConfig extends MiddlewareAuthConfig {
  isAdminApp?: boolean;
  publicRoutes?: string[];
  authRoutes?: string[];
  routePermissions?: Record<string, RoutePermissionConfig>;
  loginRedirect?: string;
  unauthorizedRedirect?: string;
  skipStaticAssets?: boolean;
}

const DEFAULT_PUBLIC_ROUTES = ['/', '/pricing', '/features', '/contact', '/privacy', '/terms'];
const DEFAULT_AUTH_ROUTES = ['/login', '/register', '/signup'];

/**
 * Creates a unified middleware function that handles both frontend and admin authentication
 */
export function createUnifiedMiddleware(config: UnifiedMiddlewareConfig = {}) {
  const {
    isAdminApp = false,
    publicRoutes = DEFAULT_PUBLIC_ROUTES,
    authRoutes = DEFAULT_AUTH_ROUTES,
    routePermissions = {},
    loginRedirect = '/login',
    unauthorizedRedirect = '/unauthorized',
    skipStaticAssets = true,
    ...authConfig
  } = config;

  return async function middleware(request: NextRequest) {
    const { pathname } = request.nextUrl;
    
    // Skip middleware for static assets and Next.js internals
    if (skipStaticAssets && (
      pathname.startsWith('/_next/') ||
      pathname.startsWith('/favicon.ico') ||
      pathname.includes('.') ||
      pathname.startsWith('/public/')
    )) {
      return NextResponse.next();
    }

    // Create base response
    let response = NextResponse.next();

    // Apply appropriate security headers
    if (isAdminApp) {
      response = addAdminSecurityHeaders(response);
    } else {
      response = addSecurityHeaders(response);
    }

    // Check if route is public (no authentication required)
    const isPublicRoute = publicRoutes.some(route => 
      pathname === route || pathname.startsWith(route + '/')
    );

    const isAuthRoute = authRoutes.includes(pathname);

    // Get session token
    const { token, isAdmin } = getSessionToken(request, authConfig);

    // For admin apps, ensure we only allow admin tokens
    if (isAdminApp && token && !isAdmin) {
      return NextResponse.redirect(new URL(unauthorizedRedirect, request.url));
    }

    // Handle authentication flow
    if (!token) {
      // No session token found
      if (isPublicRoute || isAuthRoute) {
        return response; // Allow access to public and auth routes
      }
      
      // Redirect to login for protected routes
      return NextResponse.redirect(new URL(loginRedirect, request.url));
    }

    // Handle authenticated users visiting auth routes
    if (isAuthRoute && token) {
      const redirectTo = isAdminApp ? '/' : '/dashboard';
      return NextResponse.redirect(new URL(redirectTo, request.url));
    }

    // Check permissions for protected routes
    if (!isPublicRoute && Object.keys(routePermissions).length > 0) {
      try {
        const permissionResult = await checkPermissionAccess(
          token,
          pathname,
          routePermissions,
          isAdminApp,
          authConfig
        );

        if (!permissionResult.allowed) {
          console.warn(`Access denied for ${pathname}:`, permissionResult.reason);
          
          // Redirect to appropriate error page based on the reason
          if (permissionResult.reason?.includes('authentication failed')) {
            return NextResponse.redirect(new URL(loginRedirect, request.url));
          } else {
            return NextResponse.redirect(new URL(unauthorizedRedirect, request.url));
          }
        }

        // Add user context to headers for downstream consumption
        if (permissionResult.userRole) {
          response.headers.set('x-user-role', permissionResult.userRole);
        }
        if (permissionResult.userPermissions) {
          response.headers.set('x-user-permissions', JSON.stringify(permissionResult.userPermissions));
        }
        if (permissionResult.userProfiles) {
          response.headers.set('x-user-profiles', JSON.stringify(permissionResult.userProfiles));
        }
      } catch (error) {
        console.error('Permission check failed:', error);
        // Allow access on error to prevent blocking users due to technical issues
        return response;
      }
    }

    return response;
  };
}

/**
 * Pre-configured middleware for frontend app
 */
export function createFrontendMiddleware(routePermissions: Record<string, RoutePermissionConfig> = {}) {
  return createUnifiedMiddleware({
    isAdminApp: false,
    routePermissions,
    publicRoutes: [
      '/', 
      '/pricing', 
      '/features', 
      '/contact', 
      '/privacy', 
      '/terms', 
      '/rankings', 
      '/verify-email', 
      '/access-denied', 
      '/unauthorized',
      '/dashboard', // Allow dashboard access, permission check handles the rest
      '/my-data',
      '/settings'
    ],
    authRoutes: ['/login', '/register', '/signup', '/forgot-password', '/reset-password'],
    loginRedirect: '/login',
    unauthorizedRedirect: '/unauthorized',
  });
}

/**
 * Pre-configured middleware for admin app
 */
export function createAdminMiddleware(routePermissions: Record<string, RoutePermissionConfig> = {}) {
  return createUnifiedMiddleware({
    isAdminApp: true,
    routePermissions,
    publicRoutes: [], // Admin app has no public routes
    authRoutes: ['/login'],
    loginRedirect: '/login',
    unauthorizedRedirect: '/access-denied',
    sessionCookieName: 'sess_id',
    adminSessionCookieName: 'sess_id', // Use unified session cookie
  });
}