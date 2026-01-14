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

    /**
     * Cookie name for the access token.
     * Default: 'epsx.access_token'
     */
    /**
     * URL of the backend server.
     * Required for proxying /api/proxy requests.
     */
    backendUrl?: string;
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
    const backendUrl = config.backendUrl;

    return async function authMiddleware(request: NextRequest) {
        const { pathname, search } = request.nextUrl;
        const startTime = performance.now();

        // 1. Virtual Proxy Handling (Client-Side Fetch Support)
        // Intercepts requests to /api/proxy/* and rewrites them to the backend with auth headers
        if (pathname.startsWith('/api/proxy') && backendUrl) {
            // Extract the path after /api/proxy
            // Example: /api/proxy/analytics/market-data -> /analytics/market-data
            const targetPath = pathname.replace(/^\/api\/proxy/, '');

            // Construct target URL
            const targetUrl = new URL(targetPath + search, backendUrl);

            // Create rewrite response
            const response = NextResponse.rewrite(targetUrl);

            // Copy headers from original request
            const requestHeaders = new Headers(request.headers);

            // Get token from cookies
            const token = request.cookies.get(tokenCookieName)?.value;

            // Inject Authorization header if token exists
            if (token) {
                requestHeaders.set('Authorization', `Bearer ${token}`);
            }

            // Set modified headers on the rewrite response
            // Note: In Next.js middleware, setting headers on NextResponse.rewrite passes them to the upstream
            // We need to pass the headers in the options of the rewrite, but NextResponse.rewrite takes a URL.
            // Actually, to modify headers sent to the upstream in a rewrite, we must set them on the *request* passed to rewrite,
            // or return a response with request headers modifed.
            // Correct way for Middleware Rewrite with Header Modification:

            return NextResponse.rewrite(targetUrl, {
                request: {
                    headers: requestHeaders,
                },
            });
        }

        // 2. Initialize Response with Security Headers
        const response = NextResponse.next();

        // Security Headers
        response.headers.set('x-pathname', pathname);
        response.headers.set('X-Content-Type-Options', 'nosniff');
        response.headers.set('X-Frame-Options', 'DENY');
        response.headers.set('X-XSS-Protection', '1; mode=block');
        response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
        response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
        response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

        // 3. Check for Token
        const token = request.cookies.get(tokenCookieName)?.value;
        const isAuthenticated = !!token;

        // DEBUG: Log authentication state
        console.log('🔒 Middleware Auth Check:', {
            pathname,
            tokenCookieName,
            hasToken: !!token,
            isAuthenticated,
            allCookies: request.cookies.getAll().map(c => c.name),
        });

        // 4. Determine Route Type
        const isPublicRoute = publicRoutes.some(route => {
            if (pathname === route) return true;
            if (route === '/') return false;
            return pathname.startsWith(`${route}/`);
        });

        // DEBUG: Log route matching
        console.log('🔒 Middleware Route Check:', {
            pathname,
            isPublicRoute,
            loginPath,
            homePath,
        });

        // 5. Handle Redirects

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

        // 6. Performance Tracking (Optional)
        const elapsedTime = performance.now() - startTime;
        response.headers.set('x-middleware-performance', elapsedTime.toString());

        return response;
    };
}
