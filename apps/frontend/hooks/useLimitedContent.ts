'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';

/**
 * Limit content for unauthenticated users.
 * Returns a limited subset of data for unauthenticated users, full data for authenticated users.
 *
 * @param fullData - The full data set to potentially limit
 * @param limit - The maximum number of items to return for unauthenticated users (default: 100)
 * @returns Limited data array for unauthenticated users, full data for authenticated users
 */
export function useLimitedContent<T>(fullData: T[] | null | undefined, limit: number = 100): T[] {
    const { isAuthenticated } = useSharedAuth();

    // Return full data for authenticated users
    if (isAuthenticated) {
        return (fullData ?? []) as T[];
    }

    // Return limited data for unauthenticated users
    if (!isAuthenticated && Array.isArray(fullData)) {
        return fullData.slice(0, limit);
    }

    return [];
}

/**
 * Check if content should be limited based on authentication status.
 * Useful for conditional rendering of "sign in to see more" messages.
 *
 * @returns Object with isLimited boolean and the limit count
 */
export function useContentLimit(limit: number = 100): {
    isLimited: boolean;
    limit: number;
    canShowMore: boolean;
} {
    const { isAuthenticated } = useSharedAuth();

    return {
        isLimited: !isAuthenticated,
        limit,
        canShowMore: !isAuthenticated,
    };
}
