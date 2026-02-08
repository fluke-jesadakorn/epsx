/**
 * Frontend Auth Hook
 * 
 * Provides authentication state and permissions for the frontend platform.
 * PERMISSION REFACTOR: Granular permission enforcement is now backend-only.
 */
'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';

/**
 * Hook that provides auth state for the frontend platform
 */
export function useFrontendAuth() {
    const auth = useSharedAuth();

    // Check if user can access (Permissive for authenticated)
    const canAccess = (_required?: string | string[]): boolean => {
        return auth.isAuthenticated;
    };

    return {
        user: auth.user,
        isAuthenticated: auth.isAuthenticated,
        permissions: auth.getUserPermissions(),
        canAccess,
        // Legacy compatibility
        level: auth.isAuthenticated ? 'AUTHENTICATED' : 'ANONYMOUS',
    };
}

export default useFrontendAuth;

