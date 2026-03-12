import { createAuthMiddleware } from '@/shared/auth/middleware';
import { NextResponse } from 'next/server';

// Paths exempt from the Turnstile gate
const TURNSTILE_EXEMPT = [
    '/challenge',
    '/_next',
    '/api/',
    '/favicon.ico',
    '/robots.txt',
    '/sitemap.xml',
    '/manifest.json',
];

function isTurnstileExempt(pathname: string): boolean {
    return TURNSTILE_EXEMPT.some(p => pathname.startsWith(p));
}

const authMiddleware = createAuthMiddleware({
    publicRoutes: [
        '/', // Landing page
        '/challenge',
        '/auth',
        '/access-denied',
        '/unauthorized',
        '/terms',
        '/privacy',
        '/analytics',
        '/plans',       // Browse plans without auth
        '/portfolio',   // Browse without auth; actions gated at component level
        '/chat',        // Browse without auth; sending messages requires auth
        '/dashboard',   // Browse without auth
        '/account*',    // Actions gated at component level
        '/payment*',    // Actions gated at component level
        '/developer*',  // Actions gated at component level
        '/news*',       // Public news articles
        '/manual',
        '/upgrade',
        '/api/auth*',
        '/api/public*',
        '/developer/docs/reference',
        '/developer/docs/openapi',
        '/_next*',
        '/favicon.ico',
        '/robots.txt',
        '/sitemap.xml',
        '/manifest.json',
        '/screenshots*'
    ],
    loginPath: '/auth',
    homePath: '/'
});

export function middleware(request: Parameters<typeof authMiddleware>[0]) {
    const { pathname } = request.nextUrl;

    // Turnstile gate — check before auth middleware
    if (!isTurnstileExempt(pathname)) {
        const turnstileCookie = request.cookies.get('epsx.turnstile');
        if (turnstileCookie === undefined || turnstileCookie.value === '') {
            const url = request.nextUrl.clone();
            url.pathname = '/challenge';
            url.searchParams.set('from', pathname);
            return NextResponse.redirect(url);
        }
    }

    return authMiddleware(request);
}

export const config = {
    matcher: [
        // Pattern to match all paths that should be protected or checked
        '/((?!_next/static|_next/image|favicon.ico|api/health|logos/).*)',
    ],
};
