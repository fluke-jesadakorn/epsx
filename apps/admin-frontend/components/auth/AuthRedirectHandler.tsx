'use client';

import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useRef } from 'react';

/**
 * Global handler for 401 Unauthorized errors.
 * Listens for `auth:unauthorized` events emitted by UnifiedApiClient
 * and redirects to /auth with the current path as return_url.
 */
export function AuthRedirectHandler() {
    const router = useRouter();
    const pathname = usePathname();
    const redirectingRef = useRef(false);

    useEffect(() => {
        const handleUnauthorized = (event: CustomEvent<{ returnUrl: string }>) => {
            // Prevent multiple simultaneous redirects
            if (redirectingRef.current) return;

            // Don't redirect if already on auth page
            if (pathname === '/auth' || pathname === '/login') return;

            redirectingRef.current = true;

            // Get return URL from event or use current path
            const returnUrl = event.detail?.returnUrl || pathname || '/';

            // Redirect to auth page with return URL
            const encodedReturnUrl = encodeURIComponent(returnUrl);
            router.push(`/auth?return_url=${encodedReturnUrl}&reason=no-session`);

            // Reset redirect flag after a delay to allow for future redirects
            setTimeout(() => {
                redirectingRef.current = false;
            }, 2000);
        };

        // Listen for unauthorized events
        window.addEventListener('auth:unauthorized', handleUnauthorized as EventListener);

        return () => {
            window.removeEventListener('auth:unauthorized', handleUnauthorized as EventListener);
        };
    }, [router, pathname]);

    // This component doesn't render anything
    return null;
}
