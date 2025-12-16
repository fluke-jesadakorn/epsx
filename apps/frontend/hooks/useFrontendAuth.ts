/**
 * Frontend Auth Adapter Hook
 * 
 * Bridges the frontend's shared auth system to the unified auth adapter
 * for use with UnifiedProgressiveAuthGate and UnifiedPermissionGuard.
 */
'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import type { AuthLevel, UnifiedAuthInterface } from '@/shared/components/auth/UnifiedAuthAdapter';

/**
 * Hook that provides unified auth interface for the frontend platform
 * This is used by UnifiedProgressiveAuthGate and UnifiedPermissionGuard
 */
export function useFrontendAuth(): UnifiedAuthInterface {
    const auth = useSharedAuth();

    // Get user permissions from the shared auth
    const permissions = auth.getUserPermissions();

    // Helper to check single permission
    const hasPermission = (permission: string): boolean => {
        return auth.hasPermissionForDisplay(permission);
    };

    // Helper to check any permission
    const hasAnyPermission = (permissionList: string[]): boolean => {
        return permissionList.some(p => hasPermission(p));
    };

    // Helper to check all permissions
    const hasAllPermissions = (permissionList: string[]): boolean => {
        return permissionList.every(p => hasPermission(p));
    };

    // Determine auth level based on authentication state
    const getLevel = (): AuthLevel => {
        if (!auth.isAuthenticated || !auth.user) {
            return 'ANONYMOUS';
        }
        // For now, connected wallet = AUTHENTICATED level
        // Could be extended to support PROGRESSIVE/FULL based on user tier or permissions
        return 'AUTHENTICATED';
    };

    // Check if user can access a given auth level
    const canAccess = (level: AuthLevel): boolean => {
        const levelHierarchy: Record<AuthLevel, number> = {
            'ANONYMOUS': 0,
            'AUTHENTICATED': 1,
            'PROGRESSIVE': 2,
            'FULL': 3
        };
        const currentLevel = getLevel();
        return levelHierarchy[currentLevel] >= levelHierarchy[level];
    };

    return {
        user: auth.user,
        level: getLevel(),
        permissions,
        hasPermission,
        hasAnyPermission,
        hasAllPermissions,
        canAccess
    };
}

export default useFrontendAuth;
