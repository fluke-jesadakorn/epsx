'use client';

import { useCallback } from 'react';
import type { SharedWeb3AuthClient, UnifiedApiResponse } from '../../../auth/client';
import { logger } from '../../../utils/logger';

interface UseApiHelpersProps {
    client: SharedWeb3AuthClient;
}

export function useApiHelpers({ client }: UseApiHelpersProps) {

    const getWalletAddress = useCallback(() => {
        return client.getWalletAddress();
    }, [client]);

    const getUserTier = useCallback(() => {
        return client.getUserTier();
    }, [client]);

    const getUserPermissions = useCallback(() => {
        return client.getUserPermissions();
    }, [client]);

    const makeApiRequest = useCallback(
        async <T = unknown>(
            endpoint: string,
            options?: RequestInit
        ): Promise<UnifiedApiResponse<T>> => {
            try {
                return await client.makeAuthenticatedRequest<T>(endpoint, options);
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : 'API request failed';
                logger.error('API request error', { endpoint, error: errorMessage });
                return {
                    success: false,
                    error: {
                        code: 500,
                        message: 'API request failed',
                        reason: errorMessage,
                    },
                } as UnifiedApiResponse<T>;
            }
        },
        [client]
    );

    return {
        getWalletAddress,
        getUserTier,
        getUserPermissions,
        makeApiRequest,
    };
}
