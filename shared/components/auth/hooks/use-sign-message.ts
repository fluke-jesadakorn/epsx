'use client';

import { useCallback, useState } from 'react';
import type { WalletClient } from 'viem';
import { challengeAction, loginAction, verifyAction } from '../../../auth/actions';
import type { AuthResult, AuthStep } from '../types';

interface UseSignMessageProps {
    address?: string;
    walletClient?: WalletClient | null;
    variant: 'user' | 'admin';
    turnstileToken?: string | null;
    authenticateWithDirectApi: (user: {
        wallet_address: string;
        permissions: string[];
        is_new_user: boolean;
        access_token: string;
        tier_level?: string;
    }) => Promise<void>;
    onSuccess?: (result: AuthResult) => void;
    onError?: (error: string) => void;
    onClose: () => void;
    setStep: (step: AuthStep) => void;
    setError: (error: string | null) => void;
}

export function useSignMessage({
    address,
    walletClient,
    variant,
    turnstileToken,
    authenticateWithDirectApi,
    onSuccess,
    onError,
    onClose,
    setStep,
    setError,
}: UseSignMessageProps) {
    const [isSigning, setIsSigning] = useState(false);

    const handleSign = useCallback(async () => {
        if (address === undefined || address === '' || walletClient === null || walletClient === undefined) { return; }
        try {
            setError(null);
            setStep('authenticating');
            setIsSigning(true);

            const result = await verifyAndLogin({
                address,
                walletClient,
                variant,
                turnstileToken: turnstileToken ?? undefined,
                loginAct: loginAction,
                authenticateWithDirectApi
            });

            setStep('success');
            onSuccess?.(result as AuthResult);
            onClose();
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Authentication failed';
            setError(msg);
            onError?.(msg);
            setStep('error');
        } finally {
            setIsSigning(false);
        }
    }, [address, walletClient, variant, turnstileToken, authenticateWithDirectApi, onSuccess, onError, onClose, setStep, setError]);

    return { handleSign, isSigning };
}

interface VerifyAndLoginProps {
    address: string;
    walletClient: WalletClient;
    variant: 'user' | 'admin';
    turnstileToken?: string;
    loginAct: (token: string, data: Record<string, unknown>) => Promise<{ success: boolean; error?: string }>;
    authenticateWithDirectApi: (user: {
        wallet_address: string;
        permissions: string[];
        is_new_user: boolean;
        access_token: string;
        tier_level?: string;
    }) => Promise<void>;
}

/**
 * Internal helper to handle verification and login logic
 */
async function verifyAndLogin({
    address,
    walletClient,
    variant,
    turnstileToken,
    loginAct,
    authenticateWithDirectApi
}: VerifyAndLoginProps) {
    // Server actions proxy requests through Next.js server (no direct browser→backend)
    const challengeData = await challengeAction(address, turnstileToken);

    const signature = await walletClient.signMessage({
        message: challengeData.message,
        account: address as `0x${string}`,
    });

    const result = await verifyAction({
        wallet_address: address,
        signature,
        message: challengeData.message,
        nonce: challengeData.nonce,
    });

    if (result.success !== true || result.access_token === undefined || result.access_token === '') {
        throw new Error(typeof result.error === 'string' && result.error !== '' ? result.error : 'Authentication failed');
    }

    const cookieData = {
        wallet_address: result.wallet_address,
        sub: result.wallet_address,
        auth_time: Date.now(),
        permissions: result.permissions,
        groups: variant === 'admin' ? ['admin'] : ['user'],
        isAdmin: variant === 'admin',
        expires_at: Date.now() + 2592000000,
    };

    const actionResult = await loginAct(result.access_token, cookieData);
    if (actionResult.success !== true) {
        throw new Error((typeof actionResult.error === 'string' && actionResult.error !== '') ? actionResult.error : 'Failed to save session');
    }

    await authenticateWithDirectApi({
        wallet_address: result.wallet_address,
        permissions: result.permissions,
        is_new_user: result.is_new_user,
        access_token: result.access_token,
    });

    return result;
}
