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

    // Clear expired session cookies and redirect to gate
    const params = request.nextUrl.searchParams;
    if (params.get('reason') === 'no-session' || params.has('clear')) {
        const resp = NextResponse.redirect(new URL('/', request.url));
        const isProd = process.env.NODE_ENV === 'production';
        [COOKIES.access_token, COOKIES.refresh_token, COOKIES.id_token,
         COOKIES.user, COOKIES.sid, COOKIES.auth_time, COOKIES.expires_at].forEach(name => {
            resp.cookies.set(name, '', { maxAge: 0, secure: isProd, sameSite: 'lax', path: '/' });
        });
        return resp;
    }

    // No token: pass through — page.tsx redirects to / which shows gate
    if (!hasToken) {
        return NextResponse.next();
    }

    // Has token: redirect authenticated users away from /auth
    return NextResponse.redirect(new URL('/', request.url));
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
    const { pathname } = request.nextUrl;

    // Logout via middleware — clears __Host- cookies at HTTP level
    if (request.nextUrl.searchParams.has('logout')) {
        const cleanUrl = new URL(pathname, request.url);
        const resp = NextResponse.redirect(cleanUrl);
        const isProd = process.env.NODE_ENV === 'production';
        [COOKIES.access_token, COOKIES.refresh_token, COOKIES.id_token,
         COOKIES.user, COOKIES.sid, COOKIES.auth_time, COOKIES.expires_at].forEach(name => {
            resp.cookies.set(name, '', { maxAge: 0, secure: isProd, sameSite: 'lax', path: '/' });
        });
        return resp;
    }

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

    // No token: pass through — layout gate shows auth modal inline
    if (!hasToken) {
        return NextResponse.next();
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
            // 401 = token expired → clear all auth cookies and force fresh login
            if (res.status === 401) {
                const resp = NextResponse.redirect(new URL('/auth', request.url));
                const isProd = process.env.NODE_ENV === 'production';
                const authCookies = [
                    COOKIES.access_token, COOKIES.refresh_token, COOKIES.id_token,
                    COOKIES.user, COOKIES.sid, COOKIES.auth_time, COOKIES.expires_at,
                ];
                authCookies.forEach(name => {
                    resp.cookies.set(name, '', { maxAge: 0, secure: isProd, sameSite: 'lax', path: '/' });
                });
                return resp;
            }

            const detail = await extractErrorDetail(res);
            const reason = res.status === 403 ? 'insufficient_permissions' : 'backend_error';
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
