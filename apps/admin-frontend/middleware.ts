/**
 * Enhanced Session Validation Middleware for Admin Frontend  
 * Currently disabled for stable Web3 authentication implementation
 */
import { NextRequest, NextResponse } from 'next/server';
// import { validateAdminSession } from '../../../shared/validators'; // Updated to use unified validators
import { env } from '../../shared/env/schema';

// Public routes that don't require authentication - Updated for Web3 Auth
const publicRoutes = [
  '/login', // Web3 login page
  '/api/auth/web3', // Web3 authentication endpoints
  '/api/auth/logout', 
  '/api/auth/session',
  '/api/v1', // Allow API routes to handle auth themselves
  '/api/proxy', // Allow proxy routes to handle auth themselves
  '/unauthorized',
  '/access-denied',
  '/_next',
  '/favicon.ico',
  // Web3 permission flow pages
  '/permissions/web3',
  '/request-access'
]

// Routes that require specific admin permissions - Updated for Structured Permissions
const adminPermissionRoutes: Record<string, string[]> = {
  '/users': ['admin:users:manage', 'admin:users:*', 'admin:*:*'],
  '/analytics': ['admin:analytics:view', 'admin:analytics:*', 'admin:*:*'], 
  '/settings': ['admin:system:manage', 'admin:system:*', 'admin:*:*'],
  '/permissions': ['admin:permissions:manage', 'admin:security:*', 'admin:*:*'],
  '/permission-profiles': ['admin:users:manage', 'admin:permissions:*', 'admin:*:*'],
  '/stock-ranking-packages': ['admin:content:manage', 'admin:content:*', 'admin:*:*'],
  '/reports': ['admin:analytics:view', 'admin:reports:*', 'admin:*:*'],
  '/audit': ['admin:security:view', 'admin:audit:*', 'admin:*:*'],
  '/system': ['admin:system:manage', 'admin:system:*', 'admin:*:*'],
  '/security': ['admin:security:manage', 'admin:security:*', 'admin:*:*'],
  '/notifications': ['admin:notifications:manage', 'admin:notifications:*', 'admin:*:*'],
  '/plans': ['admin:plans:manage', 'admin:content:*', 'admin:*:*'],
  '/subscriptions': ['admin:subscriptions:manage', 'admin:users:*', 'admin:*:*'],
  '/promotions': ['admin:promotions:manage', 'admin:content:*', 'admin:*:*']
}

// Performance monitoring
interface MiddlewareMetrics {
  startTime: number
  path: string
  method: string
  userAgent?: string
  ipAddress?: string
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const startTime = performance.now();
  
  // Extract request metadata for session validation
  const userAgent = request.headers.get('user-agent') || undefined;
  const ipAddress = request.headers.get('x-forwarded-for') || 
                   request.headers.get('x-real-ip') || 
                   request.ip || undefined;
  const method = request.method;
  
  // Create response with enhanced security headers
  const response = NextResponse.next();
  
  // Add comprehensive security headers for admin app
  response.headers.set('x-pathname', pathname);
  response.headers.set('x-middleware-timestamp', Date.now().toString());
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-Robots-Tag', 'noindex, nofollow');
  response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
  response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
  
  // Allow access to public routes
  const isPublicRoute = publicRoutes.some(route => 
    pathname === route || pathname.startsWith(route)
  );
  
  if (isPublicRoute) {
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());
    return response;
  }
  
  try {
    console.log(`🔍 Admin middleware: Checking authentication for - ${pathname} (${method})`);
    
    // Check for authentication using unified auth
    const { UnifiedAuth } = await import('@/lib/auth/unified-auth');
    const session = await UnifiedAuth.getSession();
    
    if (!session.isAuthenticated) {
      console.log(`🚫 Admin middleware: User not authenticated for ${pathname}`);
      return redirectToLogin(request);
    }
    
    if (!session.hasAdminAccess) {
      console.log(`🚫 Admin middleware: User lacks admin access for ${pathname}`);
      const accessDeniedUrl = new URL('/access-denied', request.url);
      accessDeniedUrl.searchParams.set('reason', 'no-admin-permissions');
      return NextResponse.redirect(accessDeniedUrl);
    }
    
    console.log(`✅ Admin middleware: Authenticated user ${session.user?.email} accessing ${pathname}`);
    
    // Authentication successful - continue with disabled permission checking for now
    /* 
    // Use new session validator service
    const validationResult = await validateAdminSession({
      userAgent,
      ipAddress,
      path: pathname,
      method
    });
    
    if (!validationResult.valid || !validationResult.user) {
      console.log(`🚫 Admin middleware: Session validation failed - ${validationResult.error}`);
      
      // Log security event for failed validation
      await logSecurityEvent({
        type: 'AUTHENTICATION_FAILED',
        userAgent,
        ipAddress,
        path: pathname,
        method,
        details: { error: validationResult.error }
      });
      
      // Redirect to login
      return redirectToLogin(request);
    }
    
    const user = validationResult.user;
    */
    
    // Permission checking disabled for Phase 4 stability
    /*
    // Start permission check timing
    const permissionCheckStartTime = performance.now();
    
    // Check for admin permission requirements
    const routePermissions = Object.entries(adminPermissionRoutes).find(([route]) => 
      pathname.startsWith(route)
    )?.[1];
    
    if (routePermissions) {
      // Check if user has any of the required permissions
      const hasAccess = routePermissions.some(permission => 
        user.permissions?.includes(permission)
      ) || user.permissions?.includes('admin:*:*');
      
      if (!hasAccess) {
        const permissionCheckTime = performance.now() - permissionCheckStartTime;
        console.log(`🚫 Admin middleware: User ${user.wallet_address} lacks required permissions ${routePermissions.join(' or ')} for ${pathname} (permission check: ${permissionCheckTime.toFixed(2)}ms)`);
        
        // Log security event for access denied
        await logSecurityEvent({
          type: 'ACCESS_DENIED',
          userId: user.id,
          userAgent,
          ipAddress,
          path: pathname,
          method,
          details: { 
            requiredPermissions: routePermissions, 
            userPermissions: user.permissions,
            permissionCheckTime: permissionCheckTime
          }
        });
        
        const accessDeniedUrl = new URL('/access-denied', request.url);
        accessDeniedUrl.searchParams.set('permissions', routePermissions.join(','));
        accessDeniedUrl.searchParams.set('route', pathname);
        return NextResponse.redirect(accessDeniedUrl);
      }
    }
    */
    
    // User info and performance tracking disabled for Phase 4 stability
    /*
    // Calculate permission check time
    const permissionCheckTime = performance.now() - permissionCheckStartTime;
    
    // Add user info to headers for server components
    response.headers.set('x-user-id', user.id);
    response.headers.set('x-user-email', user.email);
    response.headers.set('x-user-role', user.role);
    response.headers.set('x-user-permissions', JSON.stringify(user.permissions || []));
    response.headers.set('x-user-package-tier', user.package_tier);
    
    // Add performance metrics
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());
    response.headers.set('x-session-cache-hit', validationResult.performance?.cache_hit.toString() || 'false');
    response.headers.set('x-session-validation-time', validationResult.performance?.validation_time_ms.toString() || '0');
    response.headers.set('x-permission-check-time', permissionCheckTime.toString());
    
    // Log successful access with detailed performance breakdown
    if (elapsedTime > 100) { // Log slow requests
      console.warn(`⚠️  Admin middleware: Slow validation for ${pathname}: ${elapsedTime.toFixed(2)}ms (session: ${validationResult.performance?.validation_time_ms || 0}ms, permissions: ${permissionCheckTime.toFixed(2)}ms)`);
    }
    
    console.log(`✅ Admin middleware: Authenticated ${user.email} (${user.role}) accessing ${pathname} in ${elapsedTime.toFixed(2)}ms (session: ${(validationResult.performance?.validation_time_ms || 0).toFixed(2)}ms, permissions: ${permissionCheckTime.toFixed(2)}ms)`);
    */
    
    // Simple performance tracking for disabled middleware
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());
    console.log(`✅ Admin middleware: Disabled - allowing access to ${pathname} in ${elapsedTime.toFixed(2)}ms`);
    
    // Performance metrics logging disabled for Phase 4 stability
    /*
    // Log performance metrics to backend
    await recordPerformanceMetrics({
      path: pathname,
      method,
      middlewareExecutionTime: elapsedTime,
      cacheHit: validationResult.performance?.cache_hit || false,
      sessionValidationTime: validationResult.performance?.validation_time_ms || 0,
      permissionCheckTime: permissionCheckTime,
      totalRequestTime: elapsedTime
    });
    */
    
    return response;
    
  } catch (error) {
    console.error('❌ Admin middleware validation error:', error);
    const elapsedTime = performance.now() - startTime;
    
    // Log security event for middleware error
    await logSecurityEvent({
      type: 'MIDDLEWARE_ERROR',
      userAgent,
      ipAddress,
      path: pathname,
      method,
      details: { 
        error: error instanceof Error ? error.message : 'Unknown error',
        elapsedTime
      }
    });
    
    // Redirect to login on any validation error
    return redirectToLogin(request);
  }
}

/**
 * Create redirect response to login page (which will initiate wallet authentication)
 */
function redirectToLogin(request: NextRequest): NextResponse {
  // Use current request host for development services to avoid hardcoded production URLs
  const adminUrl = `${request.nextUrl.protocol}//${request.nextUrl.host}`;
  const returnUrl = `${request.nextUrl.pathname}${request.nextUrl.search}`;
  
  // Redirect to our login page which will initiate wallet authentication
  const loginUrl = new URL('/login', adminUrl);
  loginUrl.searchParams.set('return_url', returnUrl);
  loginUrl.searchParams.set('reason', 'no-session');
  
  console.log(`🔄 Admin middleware: Redirecting to wallet login: ${loginUrl.toString()}`);
  const redirect = NextResponse.redirect(loginUrl.toString());
  
  // Wallet Authentication: Clear wallet session cookies
  redirect.cookies.delete('wallet_address');
  redirect.cookies.delete('wallet_nonce');
  redirect.cookies.delete('wallet_signature');
  redirect.cookies.delete('wallet_message');
  redirect.cookies.delete('wallet_expires_at');
  
  // Also clear legacy OIDC tokens for migration
  redirect.cookies.delete('access_token');
  redirect.cookies.delete('id_token');
  redirect.cookies.delete('refresh_token');
  redirect.cookies.delete('epsx_admin_jwt');
  
  return redirect;
}

/**
 * Log security event to backend
 */
async function logSecurityEvent(event: {
  type: string
  userId?: string
  userAgent?: string
  ipAddress?: string
  path: string
  method: string
  details: Record<string, any>
}): Promise<void> {
  try {
    const backendUrl = env.BACKEND_URL;
    
    await fetch(`${backendUrl}/api/v1/security/events`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        event_type: event.type,
        severity: 'MEDIUM',
        user_id: event.userId,
        ip_address: event.ipAddress,
        user_agent: event.userAgent,
        path: event.path,
        method: event.method,
        details: event.details,
        source: 'admin-frontend-middleware'
      })
    });
  } catch (error) {
    console.error('❌ Failed to log security event:', error);
    // Don't throw - security logging is non-critical for middleware flow
  }
}

/**
 * Record performance metrics to backend
 */
async function recordPerformanceMetrics(metrics: {
  path: string
  method: string
  middlewareExecutionTime: number
  cacheHit: boolean
  sessionValidationTime: number
  permissionCheckTime: number
  totalRequestTime: number
}): Promise<void> {
  try {
    const backendUrl = env.BACKEND_URL;
    
    await fetch(`${backendUrl}/api/v1/security/metrics`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        middleware_execution_time: metrics.middlewareExecutionTime,
        cache_hit_rate: metrics.cacheHit ? 1.0 : 0.0,
        session_validation_time: metrics.sessionValidationTime,
        permission_check_time: metrics.permissionCheckTime,
        total_request_time: metrics.totalRequestTime,
        timestamp: new Date().toISOString()
      })
    });
  } catch (error) {
    console.error('❌ Failed to record performance metrics:', error);
    // Don't throw - metrics recording is non-critical for middleware flow
  }
}

export const config = {
  matcher: [
    // Enable middleware for all routes except public ones
    '/((?!api/auth|api/public|_next/static|_next/image|favicon.ico|login|unauthorized|access-denied).*)',
  ],
}