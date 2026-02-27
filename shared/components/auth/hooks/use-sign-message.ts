'use client';

import { useCallback, useRef, useState } from 'react';
import type { WalletClient } from 'viem';
import type { Config } from 'wagmi';
import { getWalletClient } from 'wagmi/actions';
import { challengeAction, loginAction, verifyAction } from '../../../auth/actions';
import { isMobile } from '../../../utils/helpers/browser';
import type { AuthResult, AuthStep } from '../types';

const sleep = (ms: number) => new Promise<void>(r => setTimeout(r, ms));

/** Wait for wagmi config to reach 'connected' status with exponential backoff */
async function waitForConnected(cfg: Config, timeoutMs?: number): Promise<void> {
    const timeout = timeoutMs ?? (isMobile() ? 25000 : 15000);
    const getStatus = () => cfg.state.status as string;
    if (getStatus() === 'connected') { return; }
    const start = Date.now();
    let delay = 300;
    while (Date.now() - start < timeout) {
        await sleep(delay);
        if (getStatus() === 'connected') { return; }
        delay = Math.min(delay * 1.5, 800);
    }
}

/** Retry getWalletClient to handle connector initialization race condition */
async function getWalletClientSafe(cfg: Config, retries = 6): Promise<WalletClient> {
    // Wait for wagmi to finish reconnecting before attempting
    await waitForConnected(cfg);

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
    const isSigningRef = useRef(false);

    const handleSign = useCallback(async () => {
        if (address === undefined || address === '') { return; }
        // Mutex: prevent concurrent sign attempts (double-click race condition)
        if (isSigningRef.current) { return; }
        isSigningRef.current = true;
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
            isSigningRef.current = false;
            setIsSigning(false);
        }
    }, [address, config, variant, turnstileToken, authenticateWithDirectApi, onSuccess, onError, onClose, setStep, setError]);

    return { handleSign, isSigning };
}

/** Parse sign/auth errors into user-friendly messages */
function parseSignError(err: unknown): string {
    const raw = err instanceof Error ? err.message : String(err);
    const lower = raw.toLowerCase();
    return matchKnownSignError(lower) ?? extractSignViemMessage(raw) ?? (raw.length > 150 ? 'Authentication failed. Please try again.' : raw);
}

function matchKnownSignError(lower: string): string | undefined {
    if (lower.includes('user rejected') || lower.includes('user denied')) {
        return 'Signature request rejected. Please approve the sign request in your wallet to log in.';
    }
    if (lower.includes('timeout') || lower.includes('timed out')) {
        return 'Signature request timed out. Please try again.';
    }
    if (lower.includes('disconnected') || lower.includes('resource not available')) {
        return 'Wallet disconnected. Please reconnect and try again.';
    }
    if (lower.includes('is not a function') || lower.includes('connector')) {
        return 'Wallet still initializing. Please try again in a moment.';
    }
    return undefined;
}

function extractSignViemMessage(raw: string): string | undefined {
    const shortMatch = raw.match(/Short Message:\s*(.+?)(?:\n|$)/i)
        ?? raw.match(/Details:\s*(.+?)(?:\n|$)/i);
    const short = shortMatch?.[1]?.trim();
    if (short !== undefined && short !== '' && short.length < 200) {
        return short;
    }
    return undefined;
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
 * Internal helper to handle verification and login logic.
 * Auto-retries once if nonce has expired.
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

    let result;
    try {
        result = await verifyAction({
            wallet_address: address,
            signature,
            message: challengeData.message,
            nonce: challengeData.nonce,
        });
    } catch (err) {
        const msg = err instanceof Error ? err.message.toLowerCase() : String(err).toLowerCase();
        const isNonceExpired = msg.includes('expired') || msg.includes('challenge not found') || msg.includes('nonce');
        if (!isNonceExpired) { throw err; }
        // Nonce expired - auto-retry with fresh challenge
        const freshChallenge = await challengeAction(address, turnstileToken);
        const freshSig = await walletClient.signMessage({
            message: freshChallenge.message,
            account: address as `0x${string}`,
        });
        result = await verifyAction({
            wallet_address: address,
            signature: freshSig,
            message: freshChallenge.message,
            nonce: freshChallenge.nonce,
        });
    }

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
