'use client';

import useSWR, { SWRConfiguration } from 'swr';

/**
 * SWR adapter for Server Actions.
 * 
 * Allows Client Components to use Server Actions with SWR's caching,
 * revalidation, and state management.
 * 
 * @param key - Unique SWR key (usually the action name + params)
 * @param action - The Server Action function
 * @param config - SWR configuration
 */
export function useServerActionSWR<T>(
    key: string | null,
    action: () => Promise<T>,
    config?: SWRConfiguration
) {
    return useSWR<T>(
        key,
        async () => {
            try {
                return await action();
            } catch (error) {
                console.error(`[useServerActionSWR] Action failed for key: ${key}`, error);
                throw error;
            }
        },
        config
    );
}
