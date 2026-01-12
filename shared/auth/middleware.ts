import { NextRequest, NextResponse } from 'next/server';

export interface AuthMiddlewareConfig {
    /**
     * List of routes that do not require authentication.
     * Supports exact matches and prefix matches.
     */
    publicRoutes: string[];

    /**
     * Path to redirect unauthenticated users to.
     * Default: '/auth'
     */
    loginPath?: string;

    /**
     * Path to redirect authenticated users to if they access public auth pages (like /login or /auth).
     * Default: '/'
     */
    homePath?: string;

    /**
     * Cookie name for the access token.
     * Default: 'epsx.access_token'
     */
    tokenCookieName?: string;
}

/**
 * Creates a middleware function to handle authentication and security headers.
 *
 * @param config Configuration for the auth middleware
 * @returns A Next.js middleware function
 */
export function createAuthMiddleware(config: AuthMiddlewareConfig) {
    const loginPath = config.loginPath || '/auth';
    const homePath = config.homePath || '/';
    const tokenCookieName = config.tokenCookieName || 'epsx.access_token';
    const publicRoutes = config.publicRoutes || [];

    return async function authMiddleware(request: NextRequest) {
        const { pathname, search } = request.nextUrl;
        const startTime = performance.now();

        // 1. Initialize Response with Security Headers
        const response = NextResponse.next();

        // Security Headers
        response.headers.set('x-pathname', pathname);
        response.headers.set('X-Content-Type-Options', 'nosniff');
        response.headers.set('X-Frame-Options', 'DENY');
        response.headers.set('X-XSS-Protection', '1; mode=block');
        response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
        response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
        response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

        // 2. Check for Token
        const token = request.cookies.get(tokenCookieName)?.value;
        const isAuthenticated = !!token;

        // 3. Determine Route Type
        const isPublicRoute = publicRoutes.some(route =>
            pathname === route || pathname.startsWith(route)
        );

        // 4. Handle Redirects

        // Case A: Unauthenticated User on Protected Route
        if (!isAuthenticated && !isPublicRoute) {
            // Create redirect URL with return_url
            const redirectUrl = new URL(loginPath, request.url);

            // Only add return_url if it's not the root path to keep URLs clean
            if (pathname !== '/') {
                redirectUrl.searchParams.set('return_url', pathname + search);
            }

            redirectUrl.searchParams.set('reason', 'no-session');

            return NextResponse.redirect(redirectUrl);
        }

        // Case B: Authenticated User on Login Page (e.g. /auth)
        // We only redirect if they are explicitly visiting the login path to prevent loops
        if (isAuthenticated && pathname === loginPath) {
            // Check for return_url in the current query params
            const returnUrl = request.nextUrl.searchParams.get('return_url');
            const targetPath = returnUrl || homePath;

            return NextResponse.redirect(new URL(targetPath, request.url));
        }

        // 5. Performance Tracking (Optional)
        const elapsedTime = performance.now() - startTime;
        response.headers.set('x-middleware-performance', elapsedTime.toString());

        return response;
    };
}
