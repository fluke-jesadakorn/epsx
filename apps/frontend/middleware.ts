import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { TokenFeature } from './types/auth/features';
import { UserRole } from './types/auth/roles';

// Feature to path mapping for protection
const PROTECTED_FEATURES: Record<string, TokenFeature> = {
  '/trading/bot': TokenFeature.TRADING_BOT,
  '/portfolio/manage': TokenFeature.PORTFOLIO_MANAGEMENT,
  '/analysis/ai': TokenFeature.AI_ANALYSIS,
  '/tools/advanced': TokenFeature.ADVANCED_TOOLS,
  '/governance': TokenFeature.GOVERNANCE,
};

// Role to minimum token balance mapping
const ROLE_MIN_TOKENS: Record<UserRole, number> = {
  [UserRole.GUEST]: 0,
  [UserRole.REGISTERED_USER]: 0,
  [UserRole.PREMIUM_USER]: 1000,
  [UserRole.TOKEN_HOLDER]: 10000,
  [UserRole.ADMINISTRATOR]: 0,
};

export async function middleware(request: NextRequest) {
  try {
    // Skip middleware for public paths
    if (request.nextUrl.pathname.startsWith('/_next') ||
        request.nextUrl.pathname.startsWith('/public') ||
        request.nextUrl.pathname === '/') {
      return NextResponse.next();
    }

    // Get auth data from cookies
    const sessionToken = request.cookies.get('__session');
    const role = request.cookies.get('role')?.value as UserRole;
    const tokenBalance = parseInt(request.cookies.get('token_balance')?.value || '0', 10);
    const features = JSON.parse(request.cookies.get('features')?.value || '[]') as TokenFeature[];

    // Check if path requires specific feature access
    for (const [path, feature] of Object.entries(PROTECTED_FEATURES)) {
      if (request.nextUrl.pathname.startsWith(path)) {
        // If not authenticated, redirect to login
        if (!sessionToken) {
          return NextResponse.redirect(new URL('/login', request.url));
        }

        // If doesn't have required feature access, redirect to upgrade
        if (!features.includes(feature)) {
          // Store attempted path for redirect after upgrade
          const response = NextResponse.redirect(new URL('/upgrade', request.url));
          response.cookies.set('intended_path', request.nextUrl.pathname);
          return response;
        }
      }
    }

    // Role-based token balance checks
    if (role && ROLE_MIN_TOKENS[role] > tokenBalance) {
      // User has role but insufficient tokens
      return NextResponse.redirect(new URL('/upgrade', request.url));
    }

    // Add user data to headers for server components
    const requestHeaders = new Headers(request.headers);
    requestHeaders.set('x-user-role', role || UserRole.GUEST);
    requestHeaders.set('x-token-balance', tokenBalance.toString());
    requestHeaders.set('x-features', JSON.stringify(features));

    return NextResponse.next({
      request: {
        headers: requestHeaders,
      },
    });
  } catch (error) {
    console.error('Middleware error:', error);
    // For security, redirect to login on any middleware errors
    return NextResponse.redirect(new URL('/login', request.url));
  }
}

// Configure paths that require middleware
export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     * - public folder
     */
    '/((?!_next/static|_next/image|favicon.ico|public).*)',
  ],
};
