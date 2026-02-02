import { decodeJwt } from 'jose';
import { NextRequest, NextResponse } from 'next/server';
import { COOKIES } from './cookies';

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
     * When enabled, never redirect - let client-side handle auth.
     * Client-side components will check auth and show modal.
     * Default: false
     */
    noRedirect?: boolean;
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
    const tokenCookieName = config.tokenCookieName || COOKIES.access_token;
    const publicRoutes = config.publicRoutes || [];
    const noRedirect = config.noRedirect || false;

    return async function authMiddleware(request: NextRequest) {
        const { pathname, search } = request.nextUrl;
        const startTime = performance.now();

        // Initialize Response with Security Headers
        const response = NextResponse.next();

        // Security Headers
        response.headers.set('x-pathname', pathname);
        response.headers.set('X-Content-Type-Options', 'nosniff');
        response.headers.set('X-Frame-Options', 'DENY');
        response.headers.set('X-XSS-Protection', '1; mode=block');
        response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
        response.headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()');
        response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

        // 3. Check for Token and Validate Expiration
        const token = request.cookies.get(tokenCookieName)?.value;
        let isAuthenticated = false;

        if (token) {
            try {
                // Decode token to check expiration (without verification signature to save time/keys)
                // We rely on the backend API to verify the signature on actual data requests
                const claims = decodeJwt(token);

                // Check expiration
                if (claims && claims.exp) {
                    const currentTime = Math.floor(Date.now() / 1000);
                    if (claims.exp > currentTime) {
                        isAuthenticated = true;
                    } else {
                        console.log('[AUTH] Middleware: Token expired', {
                            exp: claims.exp,
                            now: currentTime
                        });
                    }
                }
            } catch (e) {
                console.error('[AUTH] Middleware: Invalid token format', e);
                // Invalid token -> Not authenticated
            }
        }

        // DEBUG: Log authentication state
        console.log('[AUTH] Middleware Auth Check:', {
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
        console.log('[AUTH] Middleware Route Check:', {
            pathname,
            isPublicRoute,
            loginPath,
            homePath,
        });

        // 5. Handle Redirects

        // Case A: Unauthenticated User on Protected Route
        if (!isAuthenticated && !isPublicRoute) {
            // If noRedirect mode is enabled, let page load - client will show modal
            if (noRedirect) {
                return response;
            }

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
            // If noRedirect mode is enabled, let page load
            if (noRedirect) {
                return response;
            }

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
