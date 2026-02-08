'use client';

import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';
import { logger } from '../utils/logger';
import { COOKIES } from './cookies';

export interface AuthMiddlewareConfig {
    /** The path to redirect to for login */
    loginPath?: string;
    /** The path to redirect to after successful login if no return URL */
    homePath?: string;
    /** Cookie name for the access token */
    tokenCookieName?: string;
    /** List of public routes that don't require authentication */
    publicRoutes?: string[];
    /** If true, the middleware will not redirect, only set headers */
    noRedirect?: boolean;
}

/**
 * Creates an authentication middleware for Next.js.
 * Implements security headers and route protection.
 */
export function createAuthMiddleware(config: AuthMiddlewareConfig) {
    const loginPath = config.loginPath ?? '/auth';
    const homePath = config.homePath ?? '/';
    const tokenCookieName = config.tokenCookieName ?? COOKIES.access_token;
    const publicRoutes = config.publicRoutes;
    const noRedirect = config.noRedirect ?? false;

    return function authMiddleware(request: NextRequest) {
        const { pathname, search } = request.nextUrl;
        const startTime = performance.now();

        // 1. Initialize Response and Headers
        const response = NextResponse.next();
        applySecurityHeaders(response, pathname);

        // 2. Validate Authentication
        const token = request.cookies.get(tokenCookieName)?.value;
        const isAuthenticated = validateToken(token);

        logger.debug('[AUTH] Middleware Auth Check:', {
            pathname,
            tokenCookieName,
            hasToken: Boolean(token),
            isAuthenticated,
            allCookies: request.cookies.getAll().map(c => c.name),
        });

        // 3. Check Route Accessibility
        const isPublicRoute = checkIsPublicRoute(pathname, publicRoutes);

        logger.debug('[AUTH] Middleware Route Check:', {
            pathname,
            isPublicRoute,
            loginPath,
            homePath,
        });

        // 4. Handle Redirection Logic
        const redirectResponse = handleRedirects(
            request,
            { isAuthenticated, isPublicRoute },
            { pathname, search, loginPath, homePath, noRedirect }
        );

        if (redirectResponse) {
            return redirectResponse;
        }

        // 5. Finalize Response
        const elapsedTime = performance.now() - startTime;
        response.headers.set('x-middleware-performance', elapsedTime.toString());

        return response;
    };
}

/**
 * Orchestrates redirection logic for different authentication scenarios.
 */
function handleRedirects(
    request: NextRequest,
    authStatus: { isAuthenticated: boolean; isPublicRoute: boolean },
    context: { pathname: string; search: string; loginPath: string; homePath: string; noRedirect: boolean }
): NextResponse | null {
    const { isAuthenticated, isPublicRoute } = authStatus;
    const { pathname, search, loginPath, homePath, noRedirect } = context;

    // Case A: Unauthenticated User on Protected Route
    if (!isAuthenticated && !isPublicRoute) {
        return handleUnauthenticated({ request, pathname, search, loginPath, noRedirect });
    }

    // Case B: Explicit Login request with return_url
    if (pathname === loginPath && request.nextUrl.searchParams.has('return_url')) {
        return handleExplicitReturnUrl(request, loginPath);
    }

    // Case C: Authenticated User on Login Page
    if (isAuthenticated && pathname === loginPath) {
        return handleAuthenticatedOnLogin(request, { homePath, loginPath, noRedirect });
    }

    return null;
}

interface UnauthenticatedContext {
    request: NextRequest;
    pathname: string;
    search: string;
    loginPath: string;
    noRedirect: boolean;
}

function handleUnauthenticated(context: UnauthenticatedContext): NextResponse | null {
    const { request, pathname, search, loginPath, noRedirect } = context;
    if (noRedirect) { return null; }

    const responseRedirect = NextResponse.redirect(new URL(loginPath, request.url));
    const invalidPrefixes = ['/.well-known', '/_next', '/api', '/favicon', '/static'];
    const isValidReturnPath = !invalidPrefixes.some(p => pathname.startsWith(p));

    if (pathname !== '/' && isValidReturnPath) {
        responseRedirect.cookies.set(COOKIES.return_url, pathname + search, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            path: '/',
            maxAge: 300 // 5 minutes
        });
    }

    responseRedirect.headers.set('x-reason', 'no-session');
    return responseRedirect;
}

function handleExplicitReturnUrl(request: NextRequest, loginPath: string): NextResponse | null {
    const returnUrl = request.nextUrl.searchParams.get('return_url');
    if (returnUrl !== null && returnUrl !== '' && returnUrl !== loginPath && !returnUrl.startsWith(`${loginPath}?`)) {
        const existingReturnUrl = request.cookies.get(COOKIES.return_url)?.value;
        if (existingReturnUrl === returnUrl) {
            return null;
        }

        const cleanUrl = new URL(loginPath, request.url);
        const responseRedirect = NextResponse.redirect(cleanUrl);
        responseRedirect.cookies.set(COOKIES.return_url, returnUrl, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            path: '/',
            maxAge: 300
        });

        logger.info('[AUTH] Middleware: Setting return_url cookie and redirecting', { returnUrl });
        return responseRedirect;
    }
    return null;
}

function handleAuthenticatedOnLogin(
    request: NextRequest,
    options: { homePath: string; loginPath: string; noRedirect: boolean }
): NextResponse | null {
    const { homePath, loginPath, noRedirect } = options;
    if (noRedirect) { return null; }

    const returnUrlCookie = request.cookies.get(COOKIES.return_url)?.value;
    const returnUrlParam = request.nextUrl.searchParams.get('return_url');
    const targetPath = returnUrlCookie ?? returnUrlParam ?? homePath;

    logger.info('[AUTH] Middleware: Authenticated user on login page, redirecting', { targetPath });

    const responseRedirect = NextResponse.redirect(new URL(targetPath, request.url));
    responseRedirect.cookies.delete(COOKIES.return_url);
    return responseRedirect;
}

/**
 * Validates a JWT token.
 * Simple check for non-empty string.
 */
function validateToken(token: string | undefined): boolean {
    return token !== undefined && token !== '';
}

/**
 * Checks if a path is considered a public route.
 */
function checkIsPublicRoute(pathname: string, publicRoutes: string[] | undefined): boolean {
    if (!publicRoutes) { return false; }
    return publicRoutes.some(route => {
        if (route.endsWith('*')) {
            return pathname.startsWith(route.slice(0, -1));
        }
        return pathname === route;
    });
}

/**
 * Applies security headers to the response.
 */
function applySecurityHeaders(response: NextResponse, pathname: string) {
    const headers = response.headers;

    // Standard security headers
    headers.set('X-DNS-Prefetch-Control', 'on');
    headers.set('X-Frame-Options', 'DENY');
    headers.set('X-Content-Type-Options', 'nosniff');
    headers.set('X-XSS-Protection', '1; mode=block');
    headers.set('Referrer-Policy', 'origin-when-cross-origin');

    // Conditional caching headers
    if (pathname.includes('/api/')) {
        headers.set('Cache-Control', 'no-store, max-age=0, must-revalidate');
    }
}
