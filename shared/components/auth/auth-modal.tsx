'use client';

/**
 * SHARED AUTH MODAL COMPONENT
 * Premium 2-step auth modal: Connect Wallet -> Sign Message
 * Includes Cloudflare Turnstile CAPTCHA verification
 */

import type { Connector } from 'wagmi';
import { TurnstileWidget } from '../turnstile-widget';
import { Dialog, DialogContent } from '../ui/dialog';
import { AuthStatusDisplay, ConnectStep, SignStep, SwitchChainStep } from './auth-modal-components';
import './auth.css';
import type { AuthResult } from './hooks/use-auth-modal-logic';
import { useAuthModalLogic } from './hooks/use-auth-modal-logic';

export type { AuthResult };

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
    } = useAuthModalLogic({
        isOpen,
        variant,
        onSuccess,
        onError,
        onClose,
    });

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
            <DialogContent
                className="sm:max-w-[420px] p-0 gap-0 overflow-hidden bg-transparent dark:bg-transparent border-0 shadow-none"
                showClose={false}
            >
                <div className={`auth-modal-inner ${variant === 'admin' ? 'auth-modal-admin' : ''}`} style={{ isolation: 'isolate' }}>
                    {/* Hidden Turnstile — preloads during connect step for instant auto-sign */}
                {isOpen && turnstileToken === null && (
                    <div aria-hidden="true" style={{ position: 'absolute', opacity: 0, pointerEvents: 'none', width: 0, height: 0, overflow: 'hidden' }}>
                        <TurnstileWidget
                            onSuccess={handleTurnstileSuccess}
                            onError={handleTurnstileError}
                            onExpire={handleTurnstileExpire}
                            action="auth"
                        />
                    </div>
                )}

                <div className="auth-modal-content">
                        <AuthModalContent
                            step={step}
                            error={error}
                            isSigning={isSigning}
                            isConnecting={isConnecting}
                            isSwitching={isSwitching}
                            address={address ?? undefined}
                            connectors={connectors}
                            connect={connect}
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
            </DialogContent>
        </Dialog>
    );
}

interface AuthModalContentProps {
    step: 'connect' | 'switch-chain' | 'sign' | 'authenticating' | 'success' | 'error';
    error: string | null;
    isSigning: boolean;
    isConnecting: boolean;
    isSwitching: boolean;
    address?: string;
    connectors: readonly Connector[];
    connect: (args: { connector: Connector }) => void;
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
    address,
    connectors,
    connect,
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
