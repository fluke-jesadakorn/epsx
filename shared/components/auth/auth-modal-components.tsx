'use client';

import type { Connector } from 'wagmi';

export interface ConnectStepProps {
    connectors: readonly Connector[];
    connect: (args: { connector: Connector }) => void;
    isConnecting: boolean;
    error: string | null;
}

export function ConnectStep({ connectors, connect, isConnecting, error }: ConnectStepProps) {
    // Deduplicate connectors by name (RainbowKit registers multiple WalletConnect instances)
    const unique = connectors.filter((c, i, arr) => arr.findIndex((x) => x.name === c.name) === i);

    return (
        <div className="auth-step auth-step-enter">
            <div className="auth-step-header">
                <span className="auth-step-number">1</span>
                <span className="auth-step-label">Select Wallet</span>
            </div>
            <div className="auth-wallets">
                {unique.map((connector) => (
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

            {error !== null && error !== '' && (
                <div className="auth-connection-error">
                    <div className="auth-error-title">
                        <span>⚠️</span>
                        <span>Connection Failed</span>
                    </div>
                    <p className="auth-error-desc">{error}</p>
                </div>
            )}
        </div>
    );
}

export interface SwitchChainStepProps {
    variant: 'user' | 'admin';
    handleSwitchChain: () => void;
    isSwitching: boolean;
}

export function SwitchChainStep({ variant, handleSwitchChain, isSwitching }: SwitchChainStepProps) {
    return (
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
    );
}

export interface SignStepProps {
    address: string;
    handleSign: () => void;
    handleDisconnect: () => void;
    isSigning: boolean;
    isWalletClientLoading: boolean;
    hasWalletClient: boolean;
}

export function SignStep({
    address,
    handleSign,
    handleDisconnect,
    isSigning,
    isWalletClientLoading,
    hasWalletClient
}: SignStepProps) {
    return (
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
                disabled={isSigning || isWalletClientLoading || !hasWalletClient}
            >
                {isSigning ? (
                    <>
                        <span className="auth-spinner" />
                        Sign in Wallet...
                    </>
                ) : isWalletClientLoading || !hasWalletClient ? (
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
    );
}

export interface AuthStatusProps {
    step: 'authenticating' | 'success' | 'error';
    error?: string | null;
    handleRetry?: () => void;
    handleDisconnect?: () => void;
}

export function AuthStatusDisplay({ step, error, handleRetry, handleDisconnect }: AuthStatusProps) {
    if (step === 'authenticating') {
        return (
            <div className="auth-step auth-step-enter">
                <div className="auth-loading">
                    <span className="auth-spinner-large" />
                    <p>Verifying your identity...</p>
                </div>
            </div>
        );
    }

    if (step === 'success') {
        return (
            <div className="auth-step auth-step-enter">
                <div className="auth-success">
                    <span className="auth-success-icon">✅</span>
                    <h3>Authentication Successful!</h3>
                    <p>Redirecting...</p>
                </div>
            </div>
        );
    }

    return (
        <div className="auth-step auth-step-enter">
            <div className="auth-error">
                <span className="auth-error-icon">❌</span>
                <h3>Authentication Failed</h3>
                <p className="auth-error-msg">{error}</p>
                {handleRetry && (
                    <button className="auth-btn-primary" onClick={handleRetry}>
                        Try Again
                    </button>
                )}
                {handleDisconnect && (
                    <button className="auth-btn-secondary" onClick={handleDisconnect}>
                        Use Different Wallet
                    </button>
                )}
            </div>
        </div>
    );
}
