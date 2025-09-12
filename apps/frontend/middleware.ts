/**
 * Enhanced Session Validation Middleware for Frontend (Trading Platform)
 * Integrates with backend session validation API for 100% route coverage
 * Provides comprehensive security logging, performance monitoring, and tier-based access control
 */
import { NextRequest, NextResponse } from 'next/server';
import { validateUserSession, canAccessUserPath } from '@/lib/session-validator';
import { devLog, authLogger, logger } from '@/lib/logger';

// Public routes that don't require authentication
const publicRoutes = [
  '/',
  '/register', 
  '/forgot-password',
  '/reset-password',
  '/verify-email',
  '/access-denied',
  '/unauthorized',
  '/terms',
  '/privacy',
  '/analytics',
  '/upgrade',
  '/api/auth/callback/epsx-backend',
  '/api/auth/initiate',
  '/api/auth/login',
  '/api/auth/signin',
  '/api/auth/signout',
  '/api/auth/logout',
  '/api/auth/session',
  // New versioned auth routes
  '/api/v1/auth/callback/epsx-backend',
  '/api/v1/auth/initiate',
  '/api/v1/auth/sessions',
  '/api/v1/auth/users',
  '/api/v1/auth/user',
  '/api/v1/auth/tokens/refresh',
  '/api/v1/validations/emails',
  '/api/v1/validations/passwords',
  '/api/public', // Allow public API routes to handle auth themselves
  '/_next',
  '/favicon.ico'
]

// Premium routes that require specific package tiers
const premiumRoutes: Record<string, string> = {
  '/premium': 'BRONZE',
  '/advanced-analytics': 'BRONZE',
  '/professional': 'SILVER',
  '/alerts': 'SILVER',
  '/vip': 'GOLD',
  '/priority-support': 'GOLD',
  '/elite': 'PLATINUM',
  '/custom-dashboards': 'PLATINUM',
  '/enterprise': 'ENTERPRISE',
  '/api-access': 'ENTERPRISE'
}

// Performance monitoring interface
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
  
  // /login is now handled by API route instead of middleware
  
  // Extract request metadata for session validation
  const userAgent = request.headers.get('user-agent') || undefined;
  const ipAddress = getClientIpAddress(request);
  const method = request.method;
  
  // Create response with enhanced security headers for trading platform
  const response = NextResponse.next();
  
  // Add comprehensive security headers for trading platform
  response.headers.set('x-pathname', pathname);
  response.headers.set('x-middleware-timestamp', Date.now().toString());
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'SAMEORIGIN'); // Allow framing for charts/widgets
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-Robots-Tag', 'index, follow'); // Allow indexing for public content
  response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=(), payment=*');
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
    devLog(`Validating session for ${pathname} (${method})`);
    
    // Use session validator service
    const validationResult = await validateUserSession({
      userAgent,
      ipAddress,
      path: pathname,
      method
    });
    
    if (!validationResult.valid || !validationResult.user) {
      authLogger.warn(`Session validation failed for ${pathname}`, validationResult.error);
      
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
    
    // Check tier-based access control for premium routes
    const requiredTier = Object.entries(premiumRoutes).find(([route]) => 
      pathname.startsWith(route)
    )?.[1];
    
    if (requiredTier) {
      if (!canAccessUserPath(user, pathname)) {
        authLogger.warn(`Access denied: User ${user.email} lacks tier ${requiredTier} for ${pathname}`);
        
        // Log security event for access denied
        await logSecurityEvent({
          type: 'ACCESS_DENIED',
          userId: user.id,
          userAgent,
          ipAddress,
          path: pathname,
          method,
          details: { 
            requiredTier, 
            userTier: user.package_tier,
            requiredFeatures: pathname 
          }
        });
        
        const upgradeUrl = new URL('/upgrade', request.url);
        upgradeUrl.searchParams.set('tier', requiredTier);
        upgradeUrl.searchParams.set('feature', pathname);
        upgradeUrl.searchParams.set('current', user.package_tier);
        return NextResponse.redirect(upgradeUrl);
      }
    }
    
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
    
    // Log successful access
    if (elapsedTime > 100) { // Log slow requests
      logger.warn(`User middleware: Slow validation for ${pathname}: ${elapsedTime.toFixed(2)}ms`);
    }
    
    devLog(`Authenticated ${user.email} (${user.role}/${user.package_tier}) accessing ${pathname} in ${elapsedTime.toFixed(2)}ms`);
    
    // Log performance metrics to backend
    await recordPerformanceMetrics({
      path: pathname,
      method,
      middlewareExecutionTime: elapsedTime,
      cacheHit: validationResult.performance?.cache_hit || false,
      sessionValidationTime: validationResult.performance?.validation_time_ms || 0,
      permissionCheckTime: 0, // TODO: Measure permission check time
      totalRequestTime: elapsedTime
    });
    
    return response;
    
  } catch (error) {
    logger.error('User middleware validation error', error instanceof Error ? error.message : 'Unknown error');
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
 * Create redirect response to backend login for trading platform users
 */
function redirectToLogin(request: NextRequest): NextResponse {
  const backendUrl = process.env.NEXT_PUBLIC_API_URL || 
                    process.env.NEXT_PUBLIC_BACKEND_URL || 
                    'http://localhost:8080';
  const frontendUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000';
  const callbackUrl = `${frontendUrl}${request.nextUrl.pathname}${request.nextUrl.search}`;
  
  const loginUrl = new URL('/oauth/authorize', backendUrl);
  loginUrl.searchParams.set('client_id', 'epsx-frontend');
  loginUrl.searchParams.set('response_type', 'code');
  loginUrl.searchParams.set('scope', 'openid profile email');
  loginUrl.searchParams.set('redirect_uri', `${frontendUrl}/api/auth/callback/epsx-backend`);
  loginUrl.searchParams.set('state', Buffer.from(JSON.stringify({ redirectTo: callbackUrl })).toString('base64url'));
  
  const redirect = NextResponse.redirect(loginUrl.toString());
  
  // Clear any invalid JWT token
  redirect.cookies.delete('epsx_jwt');
  
  return redirect;
}

/**
 * Log security event to backend security API
 */
interface SecurityEventDetails {
  error?: string;
  requiredTier?: string;
  userTier?: string;
  requiredFeatures?: string;
  elapsedTime?: number;
  [key: string]: unknown;
}

async function logSecurityEvent(event: {
  type: string
  userId?: string
  userAgent?: string
  ipAddress?: string
  path: string
  method: string
  details: SecurityEventDetails
}): Promise<void> {
  try {
    const backendUrl = process.env.NEXT_PUBLIC_API_URL || 
                      process.env.NEXT_PUBLIC_BACKEND_URL || 
                      'http://localhost:8080';
    
    const response = await fetch(`${backendUrl}/api/security/events`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        event_type: event.type,
        severity: determineSeverity(event.type),
        user_id: event.userId,
        ip_address: event.ipAddress,
        user_agent: event.userAgent,
        path: event.path,
        method: event.method,
        details: event.details,
        source: 'frontend-middleware',
        timestamp: new Date().toISOString()
      })
    });
    
    if (!response.ok) {
      logger.warn(`Failed to log security event: ${response.status}`);
    }
  } catch (error) {
    logger.error('Failed to log security event', error instanceof Error ? error.message : 'Unknown error');
    // Don't throw - security logging is non-critical for middleware flow
  }
}

/**
 * Record performance metrics to backend monitoring API
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
    const backendUrl = process.env.NEXT_PUBLIC_API_URL || 
                      process.env.NEXT_PUBLIC_BACKEND_URL || 
                      'http://localhost:8080';
    
    const response = await fetch(`${backendUrl}/api/security/metrics`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        source: 'frontend-middleware',
        path: metrics.path,
        method: metrics.method,
        middleware_execution_time: metrics.middlewareExecutionTime,
        cache_hit_rate: metrics.cacheHit ? 1.0 : 0.0,
        session_validation_time: metrics.sessionValidationTime,
        permission_check_time: metrics.permissionCheckTime,
        total_request_time: metrics.totalRequestTime,
        timestamp: new Date().toISOString()
      })
    });
    
    if (!response.ok) {
      logger.warn(`Failed to record performance metrics: ${response.status}`);
    }
  } catch (error) {
    logger.error('Failed to record performance metrics', error instanceof Error ? error.message : 'Unknown error');
    // Don't throw - metrics recording is non-critical for middleware flow
  }
}

/**
 * Determine security event severity based on event type
 */
function determineSeverity(eventType: string): string {
  const criticalEvents = ['PRIVILEGE_ESCALATION_ATTEMPT', 'ACCOUNT_LOCKED', 'SUSPICIOUS_ACTIVITY'];
  const highEvents = ['AUTHENTICATION_FAILED', 'ACCESS_DENIED', 'RATE_LIMIT_EXCEEDED', 'MIDDLEWARE_ERROR'];
  const mediumEvents = ['SESSION_EXPIRED', 'INVALID_TOKEN'];
  
  if (criticalEvents.includes(eventType)) return 'CRITICAL';
  if (highEvents.includes(eventType)) return 'HIGH';
  if (mediumEvents.includes(eventType)) return 'MEDIUM';
  return 'LOW';
}

/**
 * Safely extract client IP address from NextRequest with proper type safety
 */
function getClientIpAddress(request: NextRequest): string | undefined {
  // Check headers in order of preference
  const xForwardedFor = request.headers.get('x-forwarded-for');
  if (xForwardedFor) {
    // x-forwarded-for can contain multiple IPs, take the first one
    return xForwardedFor.split(',')[0]?.trim();
  }
  
  const xRealIp = request.headers.get('x-real-ip');
  if (xRealIp) {
    return xRealIp.trim();
  }
  
  // Fallback to request.ip (may not be available in all environments)
  if ('ip' in request && request.ip) {
    return request.ip;
  }
  
  return undefined;
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api/public (public API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public (public assets)
     */
    '/((?!api/public|_next/static|_next/image|favicon.ico|public).*)',
  ],
};