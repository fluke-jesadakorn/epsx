'use client';

import { useCallback, useState } from 'react';
import type { WalletClient } from 'viem';
import type { Config } from 'wagmi';
import { getWalletClient } from 'wagmi/actions';
import { challengeAction, loginAction, verifyAction } from '../../../auth/actions';
import type { AuthResult, AuthStep } from '../types';

const sleep = (ms: number) => new Promise<void>(r => setTimeout(r, ms));

/** Wait for wagmi config to reach 'connected' status */
async function waitForConnected(cfg: Config, timeoutMs = 8000): Promise<void> {
    const getStatus = () => cfg.state.status as string;
    if (getStatus() === 'connected') return;
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
        await sleep(300);
        if (getStatus() === 'connected') return;
    }
}

/** Retry getWalletClient to handle connector initialization race condition */
async function getWalletClientSafe(cfg: Config, retries = 6): Promise<WalletClient> {
    // Wait for wagmi to finish reconnecting before attempting
    await waitForConnected(cfg, 8000);

    for (let i = 0; i < retries; i++) {
        try {
            return await getWalletClient(cfg);
        } catch (err) {
            const msg = String(err);
            if (i < retries - 1 && (msg.includes('is not a function') || msg.includes('connector'))) {
                await sleep(500 + i * 300);
                continue;
            }
            throw err;
        }
    }
    throw new Error('Wallet not ready. Please refresh and try again.');
}

interface UseSignMessageProps {
    address?: string;
    config: Config;
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
    config,
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
        if (address === undefined || address === '') { return; }
        try {
            setError(null);
            setStep('authenticating');
            setIsSigning(true);

            const walletClient = await getWalletClientSafe(config);

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
            const msg = parseSignError(err);
            setError(msg);
            onError?.(msg);
            setStep('error');
        } finally {
            setIsSigning(false);
        }
    }, [address, config, variant, turnstileToken, authenticateWithDirectApi, onSuccess, onError, onClose, setStep, setError]);

    return { handleSign, isSigning };
}

/** Parse sign/auth errors into user-friendly messages */
function parseSignError(err: unknown): string {
    const raw = err instanceof Error ? err.message : String(err);
    const lower = raw.toLowerCase();

    if (lower.includes('user rejected') || lower.includes('user denied'))
        return 'Signature request rejected. Please approve the sign request in your wallet to log in.';

    if (lower.includes('timeout') || lower.includes('timed out'))
        return 'Signature request timed out. Please try again.';

    if (lower.includes('disconnected') || lower.includes('resource not available'))
        return 'Wallet disconnected. Please reconnect and try again.';

    if (lower.includes('is not a function') || lower.includes('connector'))
        return 'Wallet still initializing. Please try again in a moment.';

    // Extract viem short message if available
    const shortMatch = raw.match(/Short Message:\s*(.+?)(?:\n|$)/i)
        ?? raw.match(/Details:\s*(.+?)(?:\n|$)/i);
    if (shortMatch?.[1]) {
        const short = shortMatch[1].trim();
        if (short.length < 200) return short;
    }

    if (raw.length > 150) return 'Authentication failed. Please try again.';

    return raw;
}

interface VerifyAndLoginProps {
    address: string;
    walletClient: WalletClient;
    variant: 'user' | 'admin';
    turnstileToken?: string;
    loginAct: (token: string, data: Record<string, unknown>, refreshToken?: string) => Promise<{ success: boolean; error?: string }>;
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

    const actionResult = await loginAct(result.access_token, cookieData, result.refresh_token);
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
