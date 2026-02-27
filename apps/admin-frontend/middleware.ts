import { COOKIES, getServerAuthToken } from '@/shared/auth/cookies';
import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

const PUBLIC_ROUTES = [
    '/login',
    '/auth',
    '/api/auth',
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
const BACKEND_URL = process.env.BACKEND_URL ?? 'http://127.0.0.1:8080';

function isPublicRoute(pathname: string): boolean {
    return PUBLIC_ROUTES.some(route => pathname.startsWith(route));
}

function handleLoginRoute(request: NextRequest, hasToken: boolean): NextResponse | null {
    if (request.nextUrl.pathname !== LOGIN_PATH) {
        return null;
    }

    if (!hasToken) {
        return NextResponse.next();
    }

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

async function extractErrorDetail(res: Response): Promise<string> {
    try {
        const text = await res.text();
        if (!text) {
            return '';
        }
        const data = JSON.parse(text) as Record<string, unknown>;
        if ('message' in data && typeof data.message === 'string') {
            return `&detail=${encodeURIComponent(data.message)}`;
        }
        if ('error' in data && typeof data.error === 'object' && data.error !== null) {
            const errObj = data.error as Record<string, unknown>;
            if ('message' in errObj && typeof errObj.message === 'string') {
                return `&detail=${encodeURIComponent(errObj.message)}`;
            }
        }
    } catch (_e) {
        // Ignore parse errors
    }
    return '';
}

/**
 * Admin-frontend middleware — authentication + authorization.
 * Verifies admin permissions by probing the backend; no JWT decoding in frontend.
 */
export async function middleware(request: NextRequest) {
    const { pathname, search } = request.nextUrl;

    const token = getServerAuthToken(request.cookies);
    const hasToken = token !== null;

    // Handle /auth (LOGIN_PATH) before public route check so authenticated users are redirected
    const loginResp = handleLoginRoute(request, hasToken);
    if (loginResp !== null) {
        return loginResp;
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

    // Verify admin permission by calling the backend — backend is the sole authority
    // Fail-closed approach: any non-2xx response rejects access
    try {
        const res = await fetch(`${BACKEND_URL}/api/admin/me`, {
            method: 'GET',
            headers: { Authorization: `Bearer ${token}` },
            signal: AbortSignal.timeout(5000),
        });

        if (!res.ok) {
            const detail = await extractErrorDetail(res);
            const reason = res.status === 401 || res.status === 403
                ? 'insufficient_permissions'
                : 'backend_error';

            return NextResponse.redirect(
                new URL(`/access-denied?reason=${reason}${detail}`, request.url)
            );
        }
    } catch {
        return NextResponse.redirect(
            new URL('/access-denied?reason=backend_unavailable', request.url)
        );
    }

    return NextResponse.next();
}

export const config = {
    matcher: [
        '/((?!_next/static|_next/image|favicon.ico|api/health).*)',
    ],
};
