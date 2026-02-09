'use client';

import { useEffect, useState } from 'react';
import type { SharedWeb3AuthClient, UserInfoResponse } from '../../../auth/client';
import { COOKIES, getClientCookie, getClientCookieJSON } from '../../../auth/cookies';
import { logger } from '../../../utils/logger';

interface UseAuthInitializationProps {
    client: SharedWeb3AuthClient;
    initialUser: UserInfoResponse | null;
    clientId: string;
    onAuthError?: (error: string) => void;
}

interface RestoreOptions {
    client: SharedWeb3AuthClient;
    clientId: string;
    setUser: (u: UserInfoResponse) => void;
    setIsLoading: (l: boolean) => void;
}

function tryRestoreFromCookies({
    client,
    clientId,
    setUser,
    setIsLoading
}: RestoreOptions): boolean {
    if (typeof window === 'undefined') { return false; }

    try {
        const clientUser = client.getCurrentUser();
        const clientHasAuth = client.isAuthenticated();

        if (clientUser !== null && clientHasAuth) {
            logger.info('[AUTH] Client successfully pre-loaded valid auth state');
            setUser(clientUser);
            setIsLoading(false);
            return true;
        }

        const storedUser = getClientCookieJSON<UserInfoResponse>(COOKIES.user);
        const accessToken = getClientCookie(COOKIES.access_token);
        const tokenExpiry = getClientCookie(COOKIES.expires_at);

        logger.info('[AUTH] Provider: Cookie restoration check', {
            clientId,
            hasStoredUser: Boolean(storedUser),
            hasAccessToken: Boolean(accessToken),
            hasTokenExpiry: Boolean(tokenExpiry),
            tokenExpiryValue: (tokenExpiry !== null && tokenExpiry !== '')
                ? new Date(parseInt(tokenExpiry, 10)).toISOString()
                : 'none',
        });

        if (storedUser !== null) {
            logger.info('[AUTH] Restoring auth from cookies', {
                clientId,
                wallet: storedUser.wallet_address.slice(0, 8),
            });
            setUser(storedUser);
            setIsLoading(false);
            return true;
        }
    } catch (storageError: unknown) {
        logger.warn('Failed to restore authentication from storage', storageError instanceof Error ? storageError.message : String(storageError));
    }
    return false;
}

export function useAuthInitialization({
    client,
    initialUser,
    clientId,
    onAuthError,
}: UseAuthInitializationProps) {
    const [user, setUser] = useState<UserInfoResponse | null>(initialUser);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const initializeAuth = async () => {
            try {
                setIsLoading(true);
                setError(null);

                let hasStoredAuth = false;

                if (initialUser !== null) {
                    logger.info('[AUTH] Provider: Hydrated from server state', {
                        wallet: initialUser.wallet_address,
                    });
                    client.setCurrentUser(initialUser);
                    hasStoredAuth = true;
                }

                if (typeof window !== 'undefined' && !hasStoredAuth) {
                    hasStoredAuth = tryRestoreFromCookies({ client, clientId, setUser, setIsLoading });
                    if (hasStoredAuth) {
                        return;
                    }
                }

                if (!hasStoredAuth) {
                    setIsLoading(false);
                    return;
                }

                const timeoutPromise = new Promise((_, reject) =>
                    setTimeout(() => reject(new Error('OpenID client timeout')), 3000)
                );

                await Promise.race([client.loadCurrentUser(), timeoutPromise]);
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : 'Failed to initialize authentication';
                logger.error('Authentication initialization failed', { error: errorMessage });
                setError(errorMessage);
                onAuthError?.(errorMessage);
            } finally {
                setIsLoading(false);
            }
        };

        const safetyTimeout = setTimeout(() => {
            setIsLoading((prev) => {
                if (prev) {
                    logger.warn('[AUTH] Provider: Initialization took too long, forcing load completion');
                    return false;
                }
                return prev;
            });
        }, 5000);

        void initializeAuth();

        const unsubscribe = client.subscribe((newUser) => {
            setUser(newUser);
            if (newUser !== null) {
                logger.info('User state updated', { wallet_address: newUser.wallet_address });
            } else {
                logger.info('User logged out');
            }
        });

        return () => {
            clearTimeout(safetyTimeout);
            unsubscribe();
        };
    }, [client, onAuthError, initialUser, clientId]);

    return {
        user,
        setUser,
        isLoading,
        setIsLoading,
        error,
        setError,
    };
}
