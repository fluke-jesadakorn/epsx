import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';
import { COOKIES } from '@/shared/auth/cookies';

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

    if (isPublicRoute(pathname)) {
        return NextResponse.next();
    }

    const token = request.cookies.get(COOKIES.access_token)?.value;

    if (token === undefined || token === '') {
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

    // Authenticated user on login page -> redirect home
    if (pathname === LOGIN_PATH) {
        const raw = request.cookies.get(COOKIES.return_url)?.value ?? '/';
        const returnUrl = isPublicRoute(raw) ? '/' : raw;
        const resp = NextResponse.redirect(new URL(returnUrl, request.url));
        resp.cookies.delete(COOKIES.return_url);
        return resp;
    }

    return NextResponse.next();
}

export const config = {
    matcher: [
        '/((?!_next/static|_next/image|favicon.ico|api/health).*)',
    ],
};
