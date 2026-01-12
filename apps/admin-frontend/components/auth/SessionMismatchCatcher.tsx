'use client';

import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export function SessionMismatchCatcher() {
    const router = useRouter();

    useEffect(() => {
        // Clear all auth cookies to break the redirect loop
        const deleteCookie = (name: string) => {
            document.cookie = `${name}=; path=/; expires=Thu, 01 Jan 1970 00:00:01 GMT;`;
            document.cookie = `${name}=; path=/; domain=${window.location.hostname}; expires=Thu, 01 Jan 1970 00:00:01 GMT;`;
        };

        // List of cookies to clear (matching those in auth.ts and middleware)
        const cookiesToClear = [
            'epsx.access_token',
            'epsx.user',
            'epsx.sid',
            'epsx.id_token',
            'epsx.refresh_token'
        ];

        cookiesToClear.forEach(deleteCookie);

        console.warn('⚠️ Session mismatch detected. Cleared cookies to prevent redirect loop.');

        // Force a hard reload via server-side logout to ensure HttpOnly cookies are also cleared
        window.location.href = '/api/auth/logout?return_url=/auth?reason=session_mismatch';
    }, [router]);

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
            <div className="text-center p-8">
                <div className="w-12 h-12 border-4 border-orange-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
                <h2 className="text-lg font-semibold text-gray-900">Refreshing Session...</h2>
                <p className="text-gray-500">Please wait while we synchronize your authentication state.</p>
            </div>
        </div>
    );
}
