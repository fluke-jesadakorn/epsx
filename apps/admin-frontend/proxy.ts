/**
 * Admin Frontend Proxy
 * Uses shared auth middleware for strict server-side protection.
 */
import { createAuthMiddleware } from '@/shared/auth/middleware';

// Configure middleware
export const middleware = createAuthMiddleware({
  publicRoutes: [
    '/login',
    '/auth',
    '/api/auth', // Web3 auth endpoints
    '/api/public',
    '/unauthorized',
    '/access-denied',
    '/_next',
    '/favicon.ico',
    '/robots.txt',
    '/sitemap.xml',
    '/manifest.json'
  ],
  loginPath: '/auth',
  homePath: '/'
});

// Rename export to match Next.js expectation if using middleware.ts, 
// but for proxy.ts in Next 16 we need to check exact export.
// Assuming "proxy" export as per previous file content, but using the middleware factory.
// We'll export 'proxy' as the main function.
export const proxy = middleware;

export const config = {
  matcher: [
    // Pattern to match all paths that should be protected or checked
    '/((?!_next/static|_next/image|favicon.ico).*)',
  ],
};