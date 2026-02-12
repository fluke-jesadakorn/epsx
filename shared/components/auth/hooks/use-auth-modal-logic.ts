import { useCallback, useEffect, useState } from 'react';
import { useAccount, useConnect, useDisconnect, useSwitchChain, useWalletClient } from 'wagmi';
import { clearClientSideCookies } from '../../../auth/cookies';
import { useSharedAuth } from '../provider';
import type { AuthResult, AuthStep } from '../types';
import { useSignMessage } from './use-sign-message';

export type { AuthResult, AuthStep } from '../types';

const BSC_MAINNET = 56;
const BSC_TESTNET = 97;
const SUPPORTED_CHAINS = [BSC_MAINNET, BSC_TESTNET];

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

    const { address, isConnected, chain, connector } = useAccount();
    const { connect, connectors, error: connectError, isPending: isConnecting } = useConnect();
    const { disconnect } = useDisconnect();
    // Only fetch wallet client when connector is fully hydrated (has methods, not just serialized stub)
    const isConnectorHydrated = isConnected && typeof connector?.getChainId === 'function';
    const { data: walletClient, isLoading: isWalletClientLoading } = useWalletClient({
        connector: isConnectorHydrated ? connector : undefined,
        query: { enabled: isConnectorHydrated, retry: false },
    });
    const { switchChainAsync, isPending: isSwitching } = useSwitchChain();
    const { authenticateWithDirectApi } = useSharedAuth();

    const isCorrectChain = chain !== undefined && SUPPORTED_CHAINS.includes(chain.id);

    // Sign message hook
    const { handleSign, isSigning } = useSignMessage({
        address,
        walletClient,
        variant,
        authenticateWithDirectApi,
        onSuccess,
        onError,
        onClose,
        setStep,
        setError,
    });

    useEffect(() => {
        if (connectError) {
            setError(connectError.message || 'Failed to connect');
        }
    }, [connectError]);

    useEffect(() => {
        if (!isOpen) { return; }

        setError(null);
        if (isConnected && address !== undefined) {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else if (walletClient) {
                setStep('sign');
            } else {
                setStep('connect');
            }
        } else {
            setStep('connect');
        }
    }, [isOpen, isConnected, address, isCorrectChain, walletClient]);

    useEffect(() => {
        if (isConnected && address !== undefined && step === 'connect') {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else if (walletClient) {
                setStep('sign');
            }
        }
    }, [isConnected, address, isCorrectChain, step, walletClient]);

    const handleSwitchChain = useCallback(async () => {
        try {
            setError(null);
            await switchChainAsync({ chainId: BSC_MAINNET });
            setStep('sign');
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Failed to switch network';
            setError(msg);
            onError?.(msg);
            setStep('error');
        }
    }, [switchChainAsync, onError]);

    const handleRetry = useCallback(() => {
        setError(null);
        if (isConnected && address !== undefined) {
            setStep(isCorrectChain ? 'sign' : 'switch-chain');
        } else {
            setStep('connect');
        }
    }, [isConnected, address, isCorrectChain]);

    const handleDisconnect = useCallback(() => {
        clearClientSideCookies();
        disconnect();
        setStep('connect');
        setError(null);
    }, [disconnect]);

    return {
        step,
        error,
        isSigning,
        isConnecting,
        isSwitching,
        isWalletClientLoading,
        address,
        connectors,
        connect,
        walletClient,
        handleSwitchChain,
        handleSign,
        handleRetry,
        handleDisconnect,
    };
}
