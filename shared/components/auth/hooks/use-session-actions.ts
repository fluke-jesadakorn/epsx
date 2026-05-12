'use client';

import { useCallback } from 'react';
import { logoutAction, refreshSessionAction } from '../../../auth/actions';
import type { SharedWeb3AuthClient } from '../../../auth/client';
import { setSharedClientToken } from '../../../utils/api-client';
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
      const result = await logoutAction();
      if (result.success === false) {
        logger.error(
          '[AUTH] Error: Failed to clear server session:',
          result.error
        );
      }
    } catch (e: unknown) {
      logger.error(
        '[AUTH] Error: Failed to clear server session:',
        e instanceof Error ? e.message : String(e)
      );
    }
  }, []);

  const logout = useCallback(async () => {
    try {
      setError(null);
      logger.info('Logging out user');

      await clearServerSession();
      client.logout();
      setSharedClientToken(undefined);
      logger.info('Logout successful');
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Logout failed';
      logger.error('Logout error', { error: errorMessage });
      setError(errorMessage);
      onAuthError?.(errorMessage);
    }
  }, [client, setError, onAuthError, clearServerSession]);

  const refreshUser = useCallback(async () => {
    try {
      setError(null);
      await client.loadCurrentUser();
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Failed to refresh user data';
      setError(errorMessage);
      onAuthError?.(errorMessage);
      throw new Error(errorMessage, { cause: err });
    }
  }, [client, setError, onAuthError]);

  const refreshSession = useCallback(async () => {
    try {
      // Use server action (can read HttpOnly refresh_token cookie)
      const result = await refreshSessionAction();
      if (result.success) {
        if (result.access_token !== undefined && result.access_token !== '') {
          client.updateTokens(result.access_token, result.expires_in);
          setSharedClientToken(result.access_token);
        }
        await client.loadCurrentUser();
        return true;
      }
      // Fallback to client-side refresh (for in-memory token)
      const clientSuccess = await client.refreshTokens();
      if (clientSuccess) {
        await client.loadCurrentUser();
      }
      return clientSuccess;
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
