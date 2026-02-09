'use client';

import { useCallback } from 'react';
import { loginAction } from '../../../auth/actions';
import type { SharedWeb3AuthClient, UserInfoResponse } from '../../../auth/client';
import { logger } from '../../../utils/logger';

interface UseWalletAuthProps {
    client: SharedWeb3AuthClient;
    setUser: (user: UserInfoResponse | null) => void;
    setIsLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
    setIsSigningChallenge: (signing: boolean) => void;
    onAuthError?: (error: string) => void;
}

/**
 * Hook to handle Web3 wallet authentication logic.
 */
export function useWalletAuth({
    client,
    setUser,
    setIsLoading,
    setError,
    setIsSigningChallenge,
    onAuthError,
}: UseWalletAuthProps) {

    const requestChallenge = useChallengeRequest({ client, setError, setIsSigningChallenge, onAuthError });
    const syncServerSession = useServerSessionSync();

    const authenticateWithWallet = useWalletSignatureAuth({
        client,
        setError,
        setIsLoading,
        onAuthError,
        syncServerSession
    });

    const authenticateWithDirectApi = useDirectApiAuth({
        setUser,
        setError,
        setIsLoading,
        onAuthError
    });

    return {
        requestChallenge,
        authenticateWithWallet,
        authenticateWithDirectApi,
    };
}

// ============================================================================
// INTERNAL HOOKS (Extracted to stay under line limits)
// ============================================================================

function useChallengeRequest({
    client,
    setError,
    setIsSigningChallenge,
    onAuthError
}: Pick<UseWalletAuthProps, 'client' | 'setError' | 'setIsSigningChallenge' | 'onAuthError'>) {
    return useCallback(
        async (walletAddress: string) => {
            try {
                setError(null);
                setIsSigningChallenge(true);
                logger.info('Requesting Web3 challenge', { wallet_address: walletAddress });

                const challenge = await client.requestChallenge(walletAddress);
                logger.info('Challenge received successfully');
                return challenge;
            } catch (err) {
                let errorMessage = 'Challenge request failed';
                if (err instanceof Error) {
                    errorMessage = err.message || errorMessage;
                    logger.error('Challenge request failed:', {
                        message: err.message,
                        backendUrl: client.getBackendUrl(),
                    });
                }
                setError(errorMessage);
                onAuthError?.(errorMessage);
                throw new Error(errorMessage);
            } finally {
                setIsSigningChallenge(false);
            }
        },
        [client, onAuthError, setError, setIsSigningChallenge]
    );
}

function useServerSessionSync() {
    return useCallback(async (user: UserInfoResponse) => {
        logger.info('Web3 authentication successful - initiating server session');
        const accessToken = user.access;

        if (typeof accessToken === 'string' && accessToken !== '') {
            const loginResult = await loginAction(accessToken, user);
            if (loginResult.success !== true) {
                const errorMsg = (typeof loginResult.error === 'string' && loginResult.error !== '') ? loginResult.error : 'Failed to create server session';
                throw new Error(errorMsg);
            }
        }
    }, []);
}

function useWalletSignatureAuth({
    client,
    setError,
    setIsLoading,
    onAuthError,
    syncServerSession
}: Pick<UseWalletAuthProps, 'client' | 'setError' | 'setIsLoading' | 'onAuthError'> & {
    syncServerSession: (user: UserInfoResponse) => Promise<void>;
}) {
    return useCallback(
        async ({
            walletAddress,
            signature,
            message,
            nonce
        }: {
            walletAddress: string;
            signature: string;
            message: string;
            nonce: string;
        }) => {
            try {
                setError(null);
                setIsLoading(true);
                logger.info('Authenticating with Web3 wallet', { wallet_address: walletAddress });

                const result = await client.authenticateWithSignature({
                    wallet_address: walletAddress,
                    signature,
                    message,
                    nonce,
                });

                if (result.success === true && result.user !== undefined) {
                    await syncServerSession(result.user);
                } else {
                    const errorMsg = (typeof result.error === 'string' && result.error !== '') ? result.error : 'Authentication failed';
                    setError(errorMsg);
                    onAuthError?.(errorMsg);
                }

                return result;
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
                logger.error('Web3 authentication error', { error: errorMessage });
                setError(errorMessage);
                onAuthError?.(errorMessage);
                return { success: false, error: errorMessage };
            } finally {
                setIsLoading(false);
            }
        },
        [client, onAuthError, setError, setIsLoading, syncServerSession]
    );
}

function useDirectApiAuth({
    setUser,
    setError,
    setIsLoading,
    onAuthError
}: Pick<UseWalletAuthProps, 'setUser' | 'setError' | 'setIsLoading' | 'onAuthError'>) {
    return useCallback(async (result: {
        wallet_address: string;
        permissions: string[];
        tier_level?: string;
        is_new_user: boolean;
        access_token?: string;
    }) => {
        await Promise.resolve();
        try {
            setError(null);
            setIsLoading(true);

            const user: UserInfoResponse = {
                sub: result.wallet_address,
                wallet_address: result.wallet_address,
                tier_level: result.tier_level ?? 'free',
                auth_method: 'web3_siwe',
                permissions: result.permissions,
                packageTier: result.tier_level ?? 'free',
                access: result.access_token,
            };

            setUser(user);
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Failed to process authentication result';
            setError(errorMessage);
            onAuthError?.(errorMessage);
            throw new Error(errorMessage);
        } finally {
            setIsLoading(false);
        }
    }, [setUser, setError, setIsLoading, onAuthError]);
}
