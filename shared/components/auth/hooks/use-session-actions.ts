'use client';

import { useCallback } from 'react';
import { logoutAction } from '../../../auth/actions';
import type { SharedWeb3AuthClient } from '../../../auth/client';
import { clearClientSideCookies } from '../../../auth/cookies';
import { logger } from '../../../utils/logger';

interface UseSessionActionsProps {
    client: SharedWeb3AuthClient;
    clientId: string;
    setError: (error: string | null) => void;
    onAuthError?: (error: string) => void;
}

export function useSessionActions({
    client,
    setError,
    onAuthError,
}: UseSessionActionsProps) {

    const clearServerSession = useCallback(async () => {
        try {
            await logoutAction();
        } catch (e: unknown) {
            if (e instanceof Error && (e.message === 'NEXT_REDIRECT' || (e as { digest?: string }).digest?.startsWith('NEXT_REDIRECT') === true)) {
                throw e;
            }
            logger.error('[AUTH] Error: Failed to clear server session:', e instanceof Error ? e.message : String(e));
        }
    }, []);

    const clearClientSession = useCallback(() => {
        if (typeof window !== 'undefined') {
            try {
                clearClientSideCookies();
            } catch (error) {
                logger.warn('[AUTH] Warning: Failed to clear authentication cookies:', error instanceof Error ? error.message : String(error));
            }
        }
    }, []);

    const logout = useCallback(async () => {
        try {
            setError(null);
            logger.info('Logging out user');

            await clearServerSession();
            clearClientSession();

            await client.logout();
            logger.info('Logout successful');
        } catch (err: unknown) {
            if (err instanceof Error && (err.message === 'NEXT_REDIRECT' || (err as { digest?: string }).digest?.startsWith('NEXT_REDIRECT') === true)) {
                throw err;
            }
            const errorMessage = err instanceof Error ? err.message : 'Logout failed';
            logger.error('Logout error', { error: errorMessage });
            setError(errorMessage);
            onAuthError?.(errorMessage);
        }
    }, [client, setError, onAuthError, clearServerSession, clearClientSession]);

    const refreshUser = useCallback(async () => {
        try {
            setError(null);
            await client.loadCurrentUser();
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : 'Failed to refresh user data';
            setError(errorMessage);
            onAuthError?.(errorMessage);
            throw new Error(errorMessage);
        }
    }, [client, setError, onAuthError]);

    const refreshSession = useCallback(async () => {
        try {
            const success = await client.refreshTokens();
            if (success) {
                await client.loadCurrentUser();
            }
            return success;
        } catch (err) {
            logger.error('[AUTH] Error: Session refresh error', err);
            return false;
        }
    }, [client]);

    return {
        logout,
        refreshUser,
        refreshSession,
    };
}
