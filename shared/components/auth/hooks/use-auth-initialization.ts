'use client';

import { useEffect, useState } from 'react';
import type { SharedWeb3AuthClient, UserInfoResponse } from '../../../auth/client';
import { refreshSessionAction } from '../../../auth/actions';
import { COOKIES } from '../../../auth/cookies';
import { setSharedClientToken } from '../../../utils/api-client';
import { logger } from '../../../utils/logger';

function getExpiresAt(): number | null {
    if (typeof document === 'undefined') { return null; }
    const name = COOKIES.expires_at;
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const match = document.cookie.match(new RegExp(`(?:^|; )${escaped}=([^;]*)`));
    if (match === null) { return null; }
    const val = parseInt(match[1], 10);
    return isNaN(val) ? null : val;
}

interface UseAuthInitializationProps {
    client: SharedWeb3AuthClient;
    initialUser: UserInfoResponse | null;
    clientId: string;
    onAuthError?: (error: string) => void;
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
                    // Share access token with all UnifiedApiClient instances
                    if (initialUser.access !== undefined && initialUser.access !== '') {
                        setSharedClientToken(initialUser.access);
                    }
                    hasStoredAuth = true;
                }

                // Try in-memory client state (no cookie reads)
                if (!hasStoredAuth && typeof window !== 'undefined') {
                    const clientUser = client.getCurrentUser();
                    if (clientUser !== null && client.isAuthenticated()) {
                        logger.info('[AUTH] Client has valid in-memory auth state');
                        setUser(clientUser);
                        setIsLoading(false);
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

    // Phase 4: Proactive token refresh - runs 5 min before expiry
    const walletAddr = user?.wallet_address ?? null;
    useEffect(() => {
        if (walletAddr === null) { return; }

        let timerId: ReturnType<typeof setTimeout> | undefined;

        const schedule = () => {
            const expiresAt = getExpiresAt();
            if (expiresAt === null) { return; }
            const delay = Math.max(expiresAt - Date.now() - 5 * 60 * 1000, 30_000);
            timerId = setTimeout(() => {
                void (async () => {
                    try {
                        const result = await refreshSessionAction();
                        if (result.success) {
                            const updated = await client.loadCurrentUser();
                            if (updated !== null) {
                                if (updated.access !== undefined) { setSharedClientToken(updated.access); }
                                setUser(updated);
                            }
                        }
                    } catch { /* refresh failed, user re-auths on expiry */ }
                    schedule();
                })();
            }, delay);
        };

        schedule();
        return () => { if (timerId !== undefined) { clearTimeout(timerId); } };
    }, [walletAddr, client]);

    return {
        user,
        setUser,
        isLoading,
        setIsLoading,
        error,
        setError,
    };
}
