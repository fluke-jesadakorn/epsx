'use client';

/**
 * SHARED AUTH MODAL COMPONENT
 * Premium 2-step auth modal: Connect Wallet → Sign Message
 */

import { createPortal } from 'react-dom';
import type { Connector } from 'wagmi';
import { AuthStatusDisplay, ConnectStep, SignStep, SwitchChainStep } from './auth-modal-components';
import './auth.css';
import type { AuthResult } from './hooks/use-auth-modal-logic';
import { useAuthModalLogic } from './hooks/use-auth-modal-logic';

export interface AuthModalProps {
    isOpen: boolean;
    onClose: () => void;
    variant: 'user' | 'admin';
    onSuccess?: (result: AuthResult) => void;
    onError?: (error: string) => void;
}

export function AuthModal({
    isOpen,
    onClose,
    variant,
    onSuccess,
    onError,
}: AuthModalProps) {
    const {
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
    } = useAuthModalLogic({
        isOpen,
        variant,
        onSuccess,
        onError,
        onClose,
    });

    if (!isOpen) { return null; }
    if (typeof document === 'undefined') { return null; }

    const title = variant === 'admin' ? '🔐 Admin Access' : '🔗 Connect Wallet';
    const subtitle = variant === 'admin' ? 'Verify your admin permissions' : 'Connect your wallet to continue';

    return createPortal(
        <div className="auth-modal-overlay" onClick={onClose}>
            <div className={`auth-modal-card ${variant === 'admin' ? 'auth-modal-admin' : ''}`} onClick={(e) => e.stopPropagation()}>
                <div className="auth-modal-header">
                    <h2 className="auth-modal-title">{title}</h2>
                    <p className="auth-modal-subtitle">{subtitle}</p>
                    <button className="auth-modal-close" onClick={onClose}>×</button>
                </div>

                <div className="auth-modal-content">
                    <AuthModalContent
                        step={step}
                        error={error}
                        isSigning={isSigning}
                        isConnecting={isConnecting}
                        isSwitching={isSwitching}
                        isWalletClientLoading={isWalletClientLoading}
                        address={address ?? undefined}
                        connectors={connectors}
                        connect={connect}
                        walletClient={walletClient}
                        handleSwitchChain={handleSwitchChain}
                        handleSign={handleSign}
                        handleRetry={handleRetry}
                        handleDisconnect={handleDisconnect}
                        variant={variant}
                    />
                </div>

                <div className="auth-modal-footer">
                    <p className="auth-footer-text">
                        {variant === 'admin'
                            ? 'Only wallets with admin permissions can access.'
                            : 'By connecting, you agree to our Terms of Service.'}
                    </p>
                </div>
            </div>
        </div>,
        document.body
    );
}

interface AuthModalContentProps {
    step: 'connect' | 'switch-chain' | 'sign' | 'authenticating' | 'success' | 'error';
    error: string | null;
    isSigning: boolean;
    isConnecting: boolean;
    isSwitching: boolean;
    isWalletClientLoading: boolean;
    address?: string;
    connectors: readonly Connector[];
    connect: (args: { connector: Connector }) => void;
    walletClient: unknown;
    handleSwitchChain: () => Promise<void>;
    handleSign: () => Promise<void>;
    handleRetry: () => void;
    handleDisconnect: () => void;
    variant: 'user' | 'admin';
}

/**
 * Internal helper to render the modal content based on current step
 */
function AuthModalContent({
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
    variant,
}: AuthModalContentProps) {
    if (step === 'connect') {
        return (
            <ConnectStep
                connectors={connectors}
                connect={connect}
                isConnecting={isConnecting}
                error={error}
            />
        );
    }

    if (step === 'switch-chain') {
        return (
            <SwitchChainStep
                variant={variant}
                handleSwitchChain={() => { void handleSwitchChain(); }}
                isSwitching={isSwitching}
            />
        );
    }

    if (step === 'sign' && typeof address === 'string' && address !== '') {
        return (
            <SignStep
                address={address}
                handleSign={() => { void handleSign(); }}
                handleDisconnect={handleDisconnect}
                isSigning={isSigning}
                isWalletClientLoading={isWalletClientLoading}
                hasWalletClient={walletClient !== null && walletClient !== undefined}
            />
        );
    }

    if (step === 'authenticating' || step === 'success' || step === 'error') {
        return (
            <AuthStatusDisplay
                step={step}
                error={error}
                handleRetry={handleRetry}
                handleDisconnect={handleDisconnect}
            />
        );
    }

    return null;
}

export default AuthModal;
