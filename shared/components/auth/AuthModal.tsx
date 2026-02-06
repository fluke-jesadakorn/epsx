'use client';

/**
 * SHARED AUTH MODAL COMPONENT
 * Premium 2-step auth modal: Connect Wallet → Sign Message
 * Features: Glassmorphism, auto chain-switch, explicit sign button
 */

import { useCallback, useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import { useAccount, useConnect, useDisconnect, useSwitchChain, useWalletClient, type Connector } from 'wagmi';
import { loginAction } from '../../auth/actions';
import { requestWalletChallenge, verifyWalletSignature } from '../../auth/api';
import { clearClientSideCookies } from '../../auth/cookies';
import './auth.css';
import { useSharedAuth } from './Provider';

// BSC Chain IDs
const BSC_MAINNET = 56;
const BSC_TESTNET = 97;
const SUPPORTED_CHAINS = [BSC_MAINNET, BSC_TESTNET];

export interface AuthModalProps {
    isOpen: boolean;
    onClose: () => void;
    variant: 'user' | 'admin';
    onSuccess?: (result: AuthResult) => void;
    onError?: (error: string) => void;
}

export interface AuthResult {
    wallet_address: string;
    permissions: string[];
    is_new_user: boolean;
    access_token: string;
}

type AuthStep = 'connect' | 'switch-chain' | 'sign' | 'authenticating' | 'success' | 'error';

export function AuthModal({
    isOpen,
    onClose,
    variant,
    onSuccess,
    onError,
}: AuthModalProps) {
    const [step, setStep] = useState<AuthStep>('connect');
    const [error, setError] = useState<string | null>(null);
    const [challenge, setChallenge] = useState<{ nonce: string; message: string } | null>(null);

    // Wagmi hooks
    const { address, isConnected, chain } = useAccount();
    const { connect, connectors, error: connectError, isPending: isConnecting } = useConnect();
    const { disconnect } = useDisconnect();
    const { data: walletClient, isLoading: isWalletClientLoading } = useWalletClient();
    const [isSigning, setIsSigning] = useState(false);
    const { switchChain, isPending: isSwitching } = useSwitchChain();

    // Shared auth context
    const { authenticateWithDirectApi, isAuthenticated } = useSharedAuth();

    // Check if on correct chain
    const isCorrectChain = chain && SUPPORTED_CHAINS.includes(chain.id);

    // Sync connection error to local state
    useEffect(() => {
        if (connectError) {
            // Clean up error message
            const msg = connectError.message || 'Failed to connect';
            // If it's the "User rejected" error, might want to show a softer message
            // or just show it as is.
            setError(msg);
        }
    }, [connectError]);

    // Reset state when modal opens
    useEffect(() => {
        if (isOpen) {
            setError(null);
            if (isConnected && address) {
                if (!isCorrectChain) {
                    setStep('switch-chain');
                } else if (walletClient) {
                    // Only go to sign if walletClient is ready
                    setStep('sign');
                } else {
                    // Wallet connected but walletClient not ready, stay on connect
                    // This prevents stuck state after sign-out
                    setStep('connect');
                }
            } else {
                setStep('connect');
            }
        }
    }, [isOpen, isConnected, address, isCorrectChain, walletClient]);

    // Handle wallet connection state changes
    // Only advance to 'sign' when wallet is connected AND walletClient is available
    useEffect(() => {
        if (isConnected && address && step === 'connect') {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else if (walletClient) {
                // Only go to sign step if walletClient is ready
                setStep('sign');
            }
            // If walletClient not ready yet, stay on 'connect' step
            // The button will show "Preparing Wallet..." via the existing logic
        }
    }, [isConnected, address, isCorrectChain, step, walletClient]);

    // Handle chain switch
    const handleSwitchChain = useCallback(async () => {
        try {
            setError(null);
            // Prefer mainnet, fallback to testnet
            const targetChain = BSC_MAINNET;
            await switchChain({ chainId: targetChain });
            setStep('sign');
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Failed to switch network';
            setError(msg);
            onError?.(msg);
            setStep('error');
        }
    }, [switchChain, onError]);

    // Handle sign message
    const handleSign = useCallback(async () => {
        if (!address || !walletClient) return;

        try {
            setError(null);
            setStep('authenticating');
            setIsSigning(true);

            // Step 1: Get challenge from backend
            const challengeData = await requestWalletChallenge(address);
            setChallenge(challengeData);

            // Step 2: Sign message with wallet
            console.log('[Auth] Signing message for address:', address);
            const signature = await walletClient.signMessage({
                message: challengeData.message,
                account: address as `0x${string}` // Explicitly pass account
            });

            // Step 3: Verify signature with backend
            const result = await verifyWalletSignature({
                wallet_address: address,
                signature,
                message: challengeData.message,
                nonce: challengeData.nonce,
            });

            if (!result.success || !result.access_token) {
                throw new Error(result.error || 'Authentication failed');
            }

            // Step 4: Persist session via server action
            const cookieData = {
                wallet_address: result.wallet_address,
                sub: result.wallet_address,
                auth_time: Date.now(),
                permissions: result.permissions || [],
                groups: variant === 'admin' ? ['admin'] : ['user'],
                isAdmin: variant === 'admin',
                expires_at: Date.now() + 2592000000, // 30 days
            };

            const actionResult = await loginAction(result.access_token, cookieData);
            if (!actionResult.success) {
                throw new Error(actionResult.error || 'Failed to save session');
            }

            // Step 5: Sync with auth provider (cookies already set by server action)
            await authenticateWithDirectApi({
                wallet_address: result.wallet_address,
                permissions: result.permissions,
                is_new_user: result.is_new_user,
                access_token: result.access_token,
            });

            setStep('success');
            onSuccess?.({
                wallet_address: result.wallet_address,
                permissions: result.permissions,
                is_new_user: result.is_new_user,
                access_token: result.access_token,
            });

            // Force page refresh after success to ensure HttpOnly cookies are sent with subsequent requests
            // This is necessary because cookies set via server action aren't immediately available
            // to client-side fetch calls until the browser reloads
            onClose();

        } catch (err) {
            console.error('[Auth] Authentication failed with error:', err);
            const msg = err instanceof Error ? err.message : 'Authentication failed';
            // Specific check for the getChainId error to give a better message
            if (msg.includes('getChainId is not a function')) {
                console.error('[Auth] Critical Connector Error: getChainId missing. Connector object:', connectors.find(c => c.id === String(chain?.id)));
            }
            setError(msg);
            onError?.(msg);
            setStep('error');
        } finally {
            setIsSigning(false);
        }
    }, [address, walletClient, variant, authenticateWithDirectApi, onSuccess, onError, onClose]);

    // Handle retry
    const handleRetry = useCallback(() => {
        setError(null);
        if (isConnected && address) {
            if (!isCorrectChain) {
                setStep('switch-chain');
            } else {
                setStep('sign');
            }
        } else {
            setStep('connect');
        }
    }, [isConnected, address, isCorrectChain]);

    // Handle disconnect
    const handleDisconnect = useCallback(() => {
        clearClientSideCookies();
        disconnect();
        setStep('connect');
        setError(null);
        setChallenge(null);
    }, [disconnect]);

    if (!isOpen) return null;

    // Use portal to avoid z-index/stacking context issues
    if (typeof document === 'undefined') return null;

    const title = variant === 'admin' ? '🔐 Admin Access' : '🔗 Connect Wallet';

    const subtitle = variant === 'admin'
        ? 'Verify your admin permissions'
        : 'Connect your wallet to continue';

    return createPortal(
        <div className="auth-modal-overlay" onClick={onClose}>
            <div className={`auth-modal-card ${variant === 'admin' ? 'auth-modal-admin' : ''}`} onClick={(e) => e.stopPropagation()}>
                {/* Header */}
                <div className="auth-modal-header">
                    <h2 className="auth-modal-title">{title}</h2>
                    <p className="auth-modal-subtitle">{subtitle}</p>
                    <button className="auth-modal-close" onClick={onClose}>×</button>
                </div>

                {/* Content */}
                <div className="auth-modal-content">
                    {/* Step: Connect */}
                    {step === 'connect' && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-step-header">
                                <span className="auth-step-number">1</span>
                                <span className="auth-step-label">Select Wallet</span>
                            </div>
                            <div className="auth-wallets">
                                {connectors.map((connector: Connector) => (
                                    <button
                                        key={connector.uid}
                                        className="auth-wallet-btn"
                                        onClick={() => connect({ connector })}
                                        disabled={isConnecting}
                                    >
                                        <span className="auth-wallet-icon">
                                            {connector.name === 'MetaMask' ? '🦊' :
                                                connector.name === 'WalletConnect' ? '🔗' : '💼'}
                                        </span>
                                        <span className="auth-wallet-name">{connector.name}</span>
                                        {isConnecting && <span className="auth-spinner" />}
                                    </button>
                                ))}
                            </div>

                            {/* Connection Error Display */}
                            {error && (
                                <div className="auth-connection-error" style={{
                                    marginTop: '1rem',
                                    padding: '0.75rem',
                                    borderRadius: '8px',
                                    backgroundColor: 'rgba(239, 68, 68, 0.1)',
                                    border: '1px solid rgba(239, 68, 68, 0.2)',
                                    color: '#ef4444',
                                    fontSize: '0.875rem',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    gap: '0.25rem'
                                }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontWeight: 600 }}>
                                        <span>⚠️</span>
                                        <span>Connection Failed</span>
                                    </div>
                                    <p style={{ margin: 0, opacity: 0.9 }}>{error}</p>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Step: Switch Chain */}
                    {step === 'switch-chain' && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-step-header">
                                <span className="auth-step-number">⚠️</span>
                                <span className="auth-step-label">Wrong Network</span>
                            </div>
                            <p className="auth-step-desc">
                                Please switch to BSC {variant === 'admin' ? 'Mainnet' : 'network'} to continue.
                            </p>
                            <button
                                className="auth-btn-primary"
                                onClick={handleSwitchChain}
                                disabled={isSwitching}
                            >
                                {isSwitching ? (
                                    <>
                                        <span className="auth-spinner" />
                                        Switching...
                                    </>
                                ) : (
                                    '🔄 Switch to BSC'
                                )}
                            </button>
                        </div>
                    )}

                    {/* Step: Sign */}
                    {step === 'sign' && address && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-step-header">
                                <span className="auth-step-number auth-step-complete">✓</span>
                                <span className="auth-step-label">Wallet Connected</span>
                            </div>
                            <p className="auth-address">
                                {address.slice(0, 6)}...{address.slice(-4)}
                            </p>
                            <div className="auth-step-header" style={{ marginTop: '1.5rem' }}>
                                <span className="auth-step-number">2</span>
                                <span className="auth-step-label">Verify Ownership</span>
                            </div>
                            <p className="auth-step-desc">
                                Sign a message to prove you own this wallet. No gas fees.
                            </p>
                            <button
                                className="auth-btn-primary"
                                onClick={handleSign}
                                disabled={isSigning || isWalletClientLoading || !walletClient}
                            >
                                {isSigning ? (
                                    <>
                                        <span className="auth-spinner" />
                                        Sign in Wallet...
                                    </>
                                ) : isWalletClientLoading || !walletClient ? (
                                    <>
                                        <span className="auth-spinner" />
                                        Preparing Wallet...
                                    </>
                                ) : (
                                    '✍️ Sign Message'
                                )}
                            </button>
                            <button className="auth-btn-secondary" onClick={handleDisconnect}>
                                Use Different Wallet
                            </button>
                        </div>
                    )}

                    {/* Step: Authenticating */}
                    {step === 'authenticating' && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-loading">
                                <span className="auth-spinner-large" />
                                <p>Verifying your identity...</p>
                            </div>
                        </div>
                    )}

                    {/* Step: Success */}
                    {step === 'success' && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-success">
                                <span className="auth-success-icon">✅</span>
                                <h3>Authentication Successful!</h3>
                                <p>Redirecting...</p>
                            </div>
                        </div>
                    )}

                    {/* Step: Error */}
                    {step === 'error' && (
                        <div className="auth-step auth-step-enter">
                            <div className="auth-error">
                                <span className="auth-error-icon">❌</span>
                                <h3>Authentication Failed</h3>
                                <p className="auth-error-msg">{error}</p>
                                <button className="auth-btn-primary" onClick={handleRetry}>
                                    Try Again
                                </button>
                                <button className="auth-btn-secondary" onClick={handleDisconnect}>
                                    Use Different Wallet
                                </button>
                            </div>
                        </div>
                    )}
                </div>

                {/* Footer */}
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

export default AuthModal;
