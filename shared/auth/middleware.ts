import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';
import { logger } from '../utils/logger';
import { COOKIES, getServerAuthToken } from './cookies';

export interface AuthMiddlewareConfig {
    /** The path to redirect to for login */
    loginPath?: string;
    /** The path to redirect to after successful login if no return URL */
    homePath?: string;
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
    const publicRoutes = config.publicRoutes;
    const noRedirect = config.noRedirect ?? false;

    return function authMiddleware(request: NextRequest) {
        const { pathname, search } = request.nextUrl;
        const startTime = performance.now();

        // 1. Initialize Response and Headers
        const response = NextResponse.next();
        applySecurityHeaders(response, pathname, request.nextUrl.hostname);

        // 2. Validate Authentication — use unified token resolution (sid → access_token → user.access)
        const token = getServerAuthToken(request.cookies);
        const isAuthenticated = token !== null;

        logger.debug('[AUTH] Middleware Auth Check:', {
            pathname,
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

    // Case C: Authenticated User on Login Page (checked before Case B so return_url redirect works)
    if (isAuthenticated && pathname === loginPath) {
        return handleAuthenticatedOnLogin(request, { homePath, loginPath, noRedirect });
    }

    // Case B: Explicit Login request with return_url (unauthenticated users only)
    if (pathname === loginPath && request.nextUrl.searchParams.has('return_url')) {
        return handleExplicitReturnUrl(request, loginPath);
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

        // Pass through (keep return_url in URL) and also set cookie as backup
        const response = NextResponse.next();
        response.cookies.set(COOKIES.return_url, returnUrl, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            path: '/',
            maxAge: 300
        });

        logger.info('[AUTH] Middleware: Setting return_url cookie', { returnUrl });
        return response;
    }
    return null;
}

function handleAuthenticatedOnLogin(
    request: NextRequest,
    options: { homePath: string; loginPath: string; noRedirect: boolean }
): NextResponse | null {
    const { homePath, noRedirect } = options;
    if (noRedirect) { return null; }

    // Break infinite redirect loop if token is expired but cookie isn't cleared
    if (request.nextUrl.searchParams.get('reason') === 'no-session' || request.nextUrl.searchParams.has('clear')) {
        const responseNext = NextResponse.next();
        const isProd = process.env.NODE_ENV === 'production';
        // Use set(maxAge=0) — delete() omits Secure flag, failing to remove __Host- cookies
        [COOKIES.access_token, COOKIES.refresh_token, COOKIES.id_token,
         COOKIES.user, COOKIES.sid, COOKIES.auth_time, COOKIES.expires_at].forEach(name => {
            responseNext.cookies.set(name, '', { maxAge: 0, secure: isProd, sameSite: 'lax', path: '/' });
        });
        return responseNext;
    }

    // If user explicitly navigated to /auth with return_url, let the page render.
    // Middleware can only check cookies, not wallet connection state (client-side).
    // The auth page handles redirect when both authenticated AND wallet connected.
    if (request.nextUrl.searchParams.has('return_url')) {
        return null;
    }

    const returnUrlCookie = request.cookies.get(COOKIES.return_url)?.value;
    const targetPath = returnUrlCookie ?? homePath;

    logger.info('[AUTH] Middleware: Authenticated user on login page, redirecting', { targetPath });

    const responseRedirect = NextResponse.redirect(new URL(targetPath, request.url));
    responseRedirect.cookies.delete(COOKIES.return_url);
    return responseRedirect;
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
function applySecurityHeaders(response: NextResponse, pathname: string, hostname = 'localhost') {
    const headers = response.headers;
    const isProd = process.env.NODE_ENV === 'production';

    // Standard security headers
    headers.set('X-DNS-Prefetch-Control', 'on');
    // Allow same-origin iframe for Scalar API docs
    const frameOpts = pathname === '/developer/docs/reference' ? 'SAMEORIGIN' : 'DENY';
    headers.set('X-Frame-Options', frameOpts);
    headers.set('X-Content-Type-Options', 'nosniff');
    headers.set('X-XSS-Protection', '1; mode=block');
    headers.set('Referrer-Policy', 'origin-when-cross-origin');
    headers.set('Permissions-Policy', 'camera=(), microphone=(), geolocation=(), payment=()');

    // HSTS - enforce HTTPS in production
    if (isProd) {
        headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
    }

    // Content Security Policy
    let devConnectSrc = '';
    if (!isProd) {
        const devSources = new Set(['http://localhost:*', 'ws://localhost:*']);
        // Add request hostname if meaningful
        if (hostname !== 'localhost' && hostname !== '0.0.0.0') {
            devSources.add(`http://${hostname}:*`);
            devSources.add(`ws://${hostname}:*`);
        }
        // Add backend URL from env (handles IP-based access like Tailscale)
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL;
        if (backendUrl !== undefined && backendUrl !== '') {
            try {
                const parsed = new URL(backendUrl);
                const wsProto = parsed.protocol === 'https:' ? 'wss:' : 'ws:';
                devSources.add(`${parsed.protocol}//${parsed.hostname}:*`);
                devSources.add(`${wsProto}//${parsed.hostname}:*`);
            } catch { /* ignore invalid URL */ }
        }
        devConnectSrc = ` ${[...devSources].join(' ')}`;
    }

    const cfScripts = 'https://challenges.cloudflare.com https://static.cloudflareinsights.com';
    const csp = [
        "default-src 'self'",
        `script-src 'self' 'unsafe-inline' 'unsafe-eval' ${cfScripts}`,
        `script-src-elem 'self' 'unsafe-inline' ${cfScripts}`,
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: blob: https:",
        "font-src 'self' data:",
        `connect-src 'self' https://*.epsx.io wss://*.epsx.io ${cfScripts} https://*.walletconnect.com wss://*.walletconnect.com https://*.walletconnect.org wss://*.walletconnect.org https://*.bnbchain.org https://*.web3modal.org${devConnectSrc}`,
        "frame-src 'self' https://challenges.cloudflare.com https://verify.walletconnect.com https://verify.walletconnect.org",
        "object-src 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ].join('; ');
    headers.set('Content-Security-Policy', csp);

    // Conditional caching headers
    if (pathname.includes('/api/')) {
        headers.set('Cache-Control', 'no-store, max-age=0, must-revalidate');
    }
}
