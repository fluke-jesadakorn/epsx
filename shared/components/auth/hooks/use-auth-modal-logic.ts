import { useCallback, useEffect, useState } from 'react';
import { useAccount, useConfig, useConnect, useDisconnect, useSwitchChain } from 'wagmi';
import { useSharedAuth } from '../provider';
import type { AuthResult, AuthStep } from '../types';
import { useSignMessage } from './use-sign-message';

export type { AuthResult, AuthStep } from '../types';

const BSC_MAINNET = 56;
const BSC_TESTNET = 97;
const SUPPORTED_CHAINS = [BSC_MAINNET, BSC_TESTNET];

/** Parse viem/wagmi errors into user-friendly messages */
function parseWalletError(err: unknown): string {
    const raw = err instanceof Error ? err.message : String(err);
    const lower = raw.toLowerCase();
    return matchKnownWalletError(lower) ?? extractViemShortMessage(raw) ?? truncateOrRaw(raw);
}

function matchKnownWalletError(lower: string): string | undefined {
    return matchWalletRejection(lower)
        ?? matchWalletProviderError(lower)
        ?? matchWalletNetworkError(lower);
}

function matchWalletRejection(lower: string): string | undefined {
    if (lower.includes('user rejected') || lower.includes('user denied') || lower.includes('rejected by user')) {
        return 'Request rejected. Please approve the request in your wallet to continue.';
    }
    if (lower.includes('already processing') || lower.includes('already pending') || lower.includes('request already pending')) {
        return 'A request is already pending in your wallet. Please check your wallet and approve or reject it.';
    }
    if (lower.includes('timeout') || lower.includes('timed out')) {
        return 'Connection timed out. Please try again.';
    }
    return undefined;
}

function matchWalletProviderError(lower: string): string | undefined {
    if (lower.includes('provider not found') || lower.includes('no provider') || lower.includes('connector not found')) {
        return 'Wallet not found. Please install the wallet extension and refresh the page.';
    }
    if (lower.includes('is not a function') || (lower.includes('connector') && !lower.includes('not found'))) {
        return 'Wallet still initializing. Please try again in a moment.';
    }
    return undefined;
}

function matchWalletNetworkError(lower: string): string | undefined {
    if (lower.includes('resource not available') || lower.includes('resourceunavailable') || lower.includes('rpc') || lower.includes('disconnected')) {
        return 'Unable to connect to the network. Please check your internet connection and wallet settings.';
    }
    if (lower.includes('chain not configured') || lower.includes('chain mismatch')) {
        return 'Network not supported. Please switch to BNB Smart Chain in your wallet.';
    }
    if (lower.includes('switch chain not supported')) {
        return 'Your wallet does not support automatic network switching. Please switch to BNB Smart Chain manually.';
    }
    return undefined;
}

function extractViemShortMessage(raw: string): string | undefined {
    const shortMatch = raw.match(/Short Message:\s*(.+?)(?:\n|$)/i)
        ?? raw.match(/Details:\s*(.+?)(?:\n|$)/i);
    const short = shortMatch?.[1]?.trim();
    if (short !== undefined && short !== '' && short.length < 200) {
        return short;
    }
    return undefined;
}

function truncateOrRaw(raw: string): string {
    if (raw.length > 150) { return 'Connection failed. Please try again or use a different wallet.'; }
    return raw;
}

interface UseAuthModalLogicProps {
    isOpen: boolean;
    variant: 'user' | 'admin';
    onSuccess?: (result: AuthResult) => void;
    onError?: (error: string) => void;
    onClose: () => void;
}

export function useAuthModalLogic({
    isOpen,
    variant,
    onSuccess,
    onError,
    onClose,
}: UseAuthModalLogicProps) {
    const [step, setStep] = useState<AuthStep>('connect');
    const [error, setError] = useState<string | null>(null);
    const [turnstileToken, setTurnstileToken] = useState<string | null>(null);

    const { address, isConnected, chain } = useAccount();
    const { connect, connectors, error: connectError, isPending: isConnecting } = useConnect();
    const { disconnect } = useDisconnect();
    const config = useConfig();
    const { switchChainAsync, isPending: isSwitching } = useSwitchChain();
    const { authenticateWithDirectApi } = useSharedAuth();

    const isCorrectChain = chain !== undefined && SUPPORTED_CHAINS.includes(chain.id);

    // Sign message hook – now receives the Turnstile token
    const { handleSign, isSigning } = useSignMessage({
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
    });

    useEffect(() => {
        if (connectError) {
            setError(parseWalletError(connectError));
        }
    }, [connectError]);

    // Reset state when modal closes
    useEffect(() => {
        if (!isOpen) {
            setTurnstileToken(null);
            setStep('connect');
            setError(null);
        }
    }, [isOpen]);

    useEffect(() => {
        if (!isOpen) { return; }

        setError(null);
        if (isConnected && address !== undefined) {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else {
                setStep('sign');
            }
        } else {
            setStep('connect');
        }
    }, [isOpen, isConnected, address, isCorrectChain]);

    useEffect(() => {
        if (!isOpen) { return; }
        if (isConnected && address !== undefined && step === 'connect') {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else {
                setStep('sign');
            }
        }
    }, [isOpen, isConnected, address, isCorrectChain, step]);

    // Auto-sign when Turnstile completes (fast path).
    useEffect(() => {
        if (!isOpen) { return; }
        if (step === 'sign' && turnstileToken !== null && !isSigning && address !== undefined) {
            void handleSign();
        }
    }, [isOpen, step, turnstileToken, isSigning, address, handleSign]);

    // Fallback: if Turnstile stalls/fails, auto-sign after 8s so user is never stuck.
    useEffect(() => {
        if (!isOpen) { return; }
        if (step !== 'sign' || isSigning || address === undefined || turnstileToken !== null) { return; }
        const timer = setTimeout(() => {
            void handleSign();
        }, 8000);
        return () => clearTimeout(timer);
    }, [isOpen, step, isSigning, address, turnstileToken, handleSign]);

    const handleSwitchChain = useCallback(async () => {
        try {
            setError(null);
            await switchChainAsync({ chainId: BSC_MAINNET });
            setStep('sign');
        } catch (err) {
            const msg = parseWalletError(err);
            setError(msg);
            onError?.(msg);
            setStep('error');
        }
    }, [switchChainAsync, onError]);

    const handleRetry = useCallback(() => {
        setError(null);
        setTurnstileToken(null);
        if (isConnected && address !== undefined) {
            setStep(isCorrectChain ? 'sign' : 'switch-chain');
        } else {
            setStep('connect');
        }
    }, [isConnected, address, isCorrectChain]);

    const handleDisconnect = useCallback(() => {
        disconnect();
        setStep('connect');
        setError(null);
        setTurnstileToken(null);
    }, [disconnect]);

    const handleTurnstileSuccess = useCallback((token: string) => { setTurnstileToken(token); }, []);
    const resetTurnstile = useCallback(() => { setTurnstileToken(null); }, []);
    const handleTurnstileError = resetTurnstile;
    const handleTurnstileExpire = resetTurnstile;

    return {
        step,
        error,
        isSigning,
        isConnecting,
        isSwitching,
        address,
        connectors,
        connect,
        handleSwitchChain,
        handleSign,
        handleRetry,
        handleDisconnect,
        turnstileToken,
        handleTurnstileSuccess,
        handleTurnstileError,
        handleTurnstileExpire,
    };
}
