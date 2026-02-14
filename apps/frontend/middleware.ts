import { createAuthMiddleware } from '@/shared/auth/middleware';

// Configure middleware for Frontend (Hybrid Mode)
export const middleware = createAuthMiddleware({
    publicRoutes: [
        '/', // Landing page
        '/auth',
        '/access-denied',
        '/unauthorized',
        '/terms',
        '/privacy',
        '/analytics',
        '/manual',
        '/upgrade',
        '/api/auth',
        '/api/public',
        '/_next',
        '/favicon.ico',
        '/robots.txt',
        '/sitemap.xml',
        '/manifest.json',
        '/screenshots*'
    ],
    loginPath: '/auth',
    homePath: '/'
});

export const config = {
    matcher: [
        // Pattern to match all paths that should be protected or checked
        '/((?!_next/static|_next/image|favicon.ico|api/health).*)',
    ],
};
