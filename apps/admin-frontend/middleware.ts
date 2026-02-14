import { createAuthMiddleware } from '@/shared/auth/middleware';

// Configure middleware for Admin Frontend
export const middleware = createAuthMiddleware({
    publicRoutes: [
        '/login',
        '/auth',
        '/api/auth', // Web3 auth endpoints
        '/manual',
        '/api/debug', // Debug endpoints
        '/api/public',
        '/unauthorized',
        '/access-denied',
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
