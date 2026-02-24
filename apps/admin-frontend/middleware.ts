import { COOKIES, getServerAuthToken } from '@/shared/auth/cookies';
import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

const PUBLIC_ROUTES = [
    '/login',
    '/auth',
    '/api/auth',
    '/manual',
    '/api/debug',
    '/api/public',
    '/unauthorized',
    '/access-denied',
    '/_next',
    '/favicon.ico',
    '/robots.txt',
    '/sitemap.xml',
    '/manifest.json',
    '/screenshots',
];

const LOGIN_PATH = '/auth';

function isPublicRoute(pathname: string): boolean {
    return PUBLIC_ROUTES.some(route => pathname.startsWith(route));
}

/**
 * Admin-frontend middleware — authentication only.
 * Backend enforces authorization via permission_validation_middleware.
 */
export function middleware(request: NextRequest) {
    const { pathname, search } = request.nextUrl;

    const token = getServerAuthToken(request.cookies);
    const hasToken = token !== null;

    // Handle /auth (LOGIN_PATH) before public route check so authenticated users are redirected
    if (pathname === LOGIN_PATH) {
        if (hasToken) {
            // Break infinite redirect loop if token is expired but cookie isn't cleared
            if (request.nextUrl.searchParams.get('reason') === 'no-session' || request.nextUrl.searchParams.has('clear')) {
                const resp = NextResponse.next();
                resp.cookies.delete(COOKIES.access_token);
                resp.cookies.delete(COOKIES.user);
                resp.cookies.delete(COOKIES.id_token);
                resp.cookies.delete(COOKIES.refresh_token);
                resp.cookies.delete(COOKIES.sid);
                return resp;
            }
            const raw = request.cookies.get(COOKIES.return_url)?.value ?? '/';
            const returnUrl = isPublicRoute(raw) ? '/' : raw;
            const resp = NextResponse.redirect(new URL(returnUrl, request.url));
            resp.cookies.delete(COOKIES.return_url);
            return resp;
        }
        return NextResponse.next();
    }

    if (isPublicRoute(pathname)) {
        return NextResponse.next();
    }

    if (!hasToken) {
        const url = new URL(LOGIN_PATH, request.url);
        const resp = NextResponse.redirect(url);
        if (pathname !== '/' && !pathname.startsWith('/_next') && !pathname.startsWith('/api')) {
            resp.cookies.set(COOKIES.return_url, pathname + search, {
                httpOnly: true,
                secure: process.env.NODE_ENV === 'production',
                sameSite: 'lax',
                path: '/',
                maxAge: 300,
            });
        }
        return resp;
    }

    return NextResponse.next();
}

export const config = {
    matcher: [
        '/((?!_next/static|_next/image|favicon.ico|api/health).*)',
    ],
};
