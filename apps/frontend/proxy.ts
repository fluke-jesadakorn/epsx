/**
 * Web3 Enterprise Authentication Middleware for Frontend
 * Validates Web3 wallet authentication and enterprise tier access
 * Provides comprehensive security logging and performance monitoring
 */
import { enterpriseUrls } from '@/config/env';
import { COOKIES } from '@/shared/auth/cookies';

// Middleware validation cache (5 seconds TTL)
const validationCache = new Map<string, { valid: boolean; user: any; timestamp: number }>();
const CACHE_TTL = 5000; // 5 seconds

// Public routes that don't require Web3 authentication
const publicRoutes = [
  '/',
  '/auth',
  '/connect-wallet',
  '/access-denied',
  '/unauthorized',
  '/terms',
  '/privacy',
  '/analytics',
  '/upgrade',
  '/api/auth',
  '/api/public',
  '/_next',
  '/favicon.ico'
]

// Enterprise tier routes that require specific access levels
const enterpriseRoutes: Record<string, string> = {
  '/starter': 'Starter',        // $1,000+ in verified tokens
  '/business': 'Business',      // $10,000+ in tokens OR enterprise NFT
  '/enterprise': 'Enterprise',  // $100,000+ in tokens OR DAO membership
  '/whale': 'Whale',           // $1,000,000+ in tokens (unlimited access)
  '/marketplace': 'Starter',    // Marketplace access
  '/billing': 'Starter',        // Billing dashboard
  '/api-access': 'Business',    // API key generation
  '/advanced-analytics': 'Business',
  '/white-label': 'Enterprise',
  '/custom-integration': 'Enterprise',
  '/priority-support': 'Whale',
  '/unlimited-access': 'Whale'
}

// Web3 Enterprise User interface
interface EnterpriseUser {
  wallet_address: string;
  enterprise_tier: string;
  permissions: string[];
  has_api_access: boolean;
  verified_tokens_usd: number;
  nft_collections: string[];
  dao_memberships: string[];
}

export async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const startTime = performance.now();

  // Extract request metadata for authentication
  const userAgent = request.headers.get('user-agent') || undefined;
  const ipAddress = getClientIpAddress(request);
  const method = request.method;

  // Create response with enhanced security headers for Web3 enterprise platform
  const response = NextResponse.next();

  // Add comprehensive security headers
  response.headers.set('x-pathname', pathname);
  response.headers.set('x-middleware-timestamp', Date.now().toString());
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'SAMEORIGIN');
  response.headers.set('X-XSS-Protection', '1; mode=block');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-Robots-Tag', 'index, follow');
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
    // Validate Web3 enterprise authentication
    const authResult = await validateWeb3Authentication(request);

    if (!authResult.valid || !authResult.user) {
      // Log security event for failed authentication
      await logSecurityEvent({
        type: 'WEB3_AUTHENTICATION_FAILED',
        userAgent,
        ipAddress,
        path: pathname,
        method,
        details: { error: authResult.error }
      });

      // Allow page to render, client-side auth will handle wallet connection
      return response;
    }

    const user = authResult.user;

    // Check enterprise tier access for protected routes
    const requiredTier = Object.entries(enterpriseRoutes).find(([route]) =>
      pathname.startsWith(route)
    )?.[1];

    if (requiredTier) {
      if (!hasEnterpriseAccess(user, requiredTier)) {
        // Log security event for access denied
        await logSecurityEvent({
          type: 'ENTERPRISE_ACCESS_DENIED',
          walletAddress: user.wallet_address,
          userAgent,
          ipAddress,
          path: pathname,
          method,
          details: {
            requiredTier,
            userTier: user.enterprise_tier,
            verifiedTokensUsd: user.verified_tokens_usd
          }
        });

        const upgradeUrl = new URL('/upgrade', request.url);
        upgradeUrl.searchParams.set('tier', requiredTier);
        upgradeUrl.searchParams.set('feature', pathname);
        upgradeUrl.searchParams.set('current', user.enterprise_tier);
        return NextResponse.redirect(upgradeUrl);
      }
    }

    // Add Web3 user info to headers for server components
    response.headers.set('x-wallet-address', user.wallet_address);
    response.headers.set('x-enterprise-tier', user.enterprise_tier);
    response.headers.set('x-verified-tokens-usd', user.verified_tokens_usd.toString());
    response.headers.set('x-has-api-access', user.has_api_access.toString());
    response.headers.set('x-permissions', JSON.stringify(user.permissions || []));

    // Add performance metrics
    const elapsedTime = performance.now() - startTime;
    response.headers.set('x-middleware-performance', elapsedTime.toString());

    // Log performance metrics to backend
    await recordPerformanceMetrics({
      path: pathname,
      method,
      middlewareExecutionTime: elapsedTime,
      walletAddress: user.wallet_address,
      enterpriseTier: user.enterprise_tier
    });

    return response;

  } catch (error) {
    console.error('Web3 middleware validation error:', error);

    // Log security event for middleware error
    await logSecurityEvent({
      type: 'MIDDLEWARE_ERROR',
      userAgent,
      ipAddress,
      path: pathname,
      method,
      details: {
        error: error instanceof Error ? error.message : 'Unknown error'
      }
    });

    // Allow page to render, client-side auth will handle errors
    return response;
  }
}


/**
 * Validate Web3 enterprise authentication
 */
async function validateWeb3Authentication(request: NextRequest): Promise<{
  valid: boolean;
  user?: EnterpriseUser;
  error?: string;
}> {
  try {
    // Extract Bearer token from Authorization header or session cookies
    const authHeader = request.headers.get('authorization');
    const bearerToken = authHeader?.startsWith('Bearer ') ? authHeader.slice(7) : null;

    // Fallback to session cookies using correct cookie names
    const accessToken = bearerToken || request.cookies.get(COOKIES.access)?.value;

    if (!accessToken) {
      return { valid: false, error: 'No authentication token found' };
    }

    // Check cache first
    const cacheKey = accessToken.substring(0, 20); // Use token prefix as cache key
    const now = Date.now();
    const cached = validationCache.get(cacheKey);

    if (cached && (now - cached.timestamp) < CACHE_TTL) {
      console.log('Using cached middleware validation');
      return { valid: cached.valid, user: cached.user };
    }

    // Validate with enterprise API
    const response = await fetch(enterpriseUrls.permissions, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const result = { valid: false, error: `Authentication validation failed: ${response.status}` };
      // Cache failed validation for shorter period (1 second)
      validationCache.set(cacheKey, { valid: false, user: undefined, timestamp: now });
      setTimeout(() => validationCache.delete(cacheKey), 1000);
      return result;
    }

    const data = await response.json();

    const user: EnterpriseUser = {
      wallet_address: data.wallet_address,
      enterprise_tier: data.enterprise_tier,
      permissions: data.permissions || [],
      has_api_access: data.has_api_access || false,
      verified_tokens_usd: data.verified_tokens_usd || 0,
      nft_collections: data.nft_collections || [],
      dao_memberships: data.dao_memberships || []
    };

    // Cache successful validation
    validationCache.set(cacheKey, { valid: true, user, timestamp: now });
    setTimeout(() => validationCache.delete(cacheKey), CACHE_TTL);

    return { valid: true, user };

  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : 'Authentication validation error'
    };
  }
}

/**
 * Check if user has access to required enterprise tier
 */
function hasEnterpriseAccess(user: EnterpriseUser, requiredTier: string): boolean {
  const tierHierarchy = {
    'Starter': 1,
    'Business': 2,
    'Enterprise': 3,
    'Whale': 4
  };

  const userLevel = tierHierarchy[user.enterprise_tier as keyof typeof tierHierarchy] || 0;
  const requiredLevel = tierHierarchy[requiredTier as keyof typeof tierHierarchy] || 1;

  return userLevel >= requiredLevel;
}

/**
 * Log security event to enterprise backend API
 */
interface SecurityEventDetails {
  error?: string;
  requiredTier?: string;
  userTier?: string;
  verifiedTokensUsd?: number;
  [key: string]: unknown;
}

async function logSecurityEvent(event: {
  type: string
  walletAddress?: string
  userAgent?: string
  ipAddress?: string
  path: string
  method: string
  details: SecurityEventDetails
}): Promise<void> {
  try {
    const response = await fetch(`${enterpriseUrls.health.replace('/health', '/security/events')}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        event_type: event.type,
        severity: determineSeverity(event.type),
        wallet_address: event.walletAddress,
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
      console.warn(`Failed to log security event: ${response.status}`);
    }
  } catch (error) {
    console.error('Failed to log security event:', error);
    // Don't throw - security logging is non-critical for middleware flow
  }
}

/**
 * Record performance metrics to enterprise backend
 */
async function recordPerformanceMetrics(metrics: {
  path: string
  method: string
  middlewareExecutionTime: number
  walletAddress: string
  enterpriseTier: string
}): Promise<void> {
  try {
    const response = await fetch(`${enterpriseUrls.health.replace('/health', '/metrics')}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        source: 'frontend-middleware',
        path: metrics.path,
        method: metrics.method,
        middleware_execution_time: metrics.middlewareExecutionTime,
        wallet_address: metrics.walletAddress,
        enterprise_tier: metrics.enterpriseTier,
        timestamp: new Date().toISOString()
      })
    });

    if (!response.ok) {
      console.warn(`Failed to record performance metrics: ${response.status}`);
    }
  } catch (error) {
    console.error('Failed to record performance metrics:', error);
    // Don't throw - metrics recording is non-critical for middleware flow
  }
}

/**
 * Determine security event severity for Web3 enterprise events
 */
function determineSeverity(eventType: string): string {
  const criticalEvents = ['WALLET_COMPROMISE_DETECTED', 'ENTERPRISE_BREACH_ATTEMPT', 'SUSPICIOUS_TRANSACTION'];
  const highEvents = ['WEB3_AUTHENTICATION_FAILED', 'ENTERPRISE_ACCESS_DENIED', 'RATE_LIMIT_EXCEEDED', 'MIDDLEWARE_ERROR'];
  const mediumEvents = ['TOKEN_EXPIRED', 'INVALID_SIGNATURE', 'TIER_INSUFFICIENT'];

  if (criticalEvents.includes(eventType)) return 'CRITICAL';
  if (highEvents.includes(eventType)) return 'HIGH';
  if (mediumEvents.includes(eventType)) return 'MEDIUM';
  return 'LOW';
}

/**
 * Safely extract client IP address from NextRequest
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