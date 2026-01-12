import { createAuthMiddleware } from '@/shared/auth/middleware';

// Configure middleware for Frontend (Hybrid Mode)
export const proxy = createAuthMiddleware({
    publicRoutes: [
        '/', // Landing page
        '/auth',
        '/connect-wallet',
        '/access-denied',
        '/unauthorized',
        '/terms',
        '/privacy',
        '/analytics',
        '/upgrade',
        '/api/auth',
        '/api/public',
        '/_next',
        '/favicon.ico',
        '/robots.txt',
        '/sitemap.xml',
        '/manifest.json'
    ],
    loginPath: '/connect-wallet',
    homePath: '/',
    backendUrl: process.env.BACKEND_URL || 'http://localhost:8080'
});

export const config = {
    matcher: [
        // Pattern to match all paths that should be protected or checked
        '/((?!_next/static|_next/image|favicon.ico).*)',
    ],
};
