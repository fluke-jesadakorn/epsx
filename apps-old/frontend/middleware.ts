import { createAuthMiddleware } from '@/shared/auth/middleware';

const authMiddleware = createAuthMiddleware({
    publicRoutes: [
        '/', // Landing page
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
    return authMiddleware(request);
}

export const config = {
    matcher: [
        // Pattern to match all paths that should be protected or checked
        '/((?!_next/static|_next/image|favicon.ico|api/health|logos/).*)',
    ],
};
