'use client';

/**
 * AUTH STATUS COMPONENT
 * Header component showing authentication state with dropdown menu
 */

import { useCallback, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useAccount, useDisconnect } from 'wagmi';
import type { UserInfoResponse } from '../../auth/client';
import { logger } from '../../utils/logger';
import { AuthModal } from './auth-modal';
import './auth.css';
import { useSharedAuth } from './provider';

export interface AuthStatusProps {
    variant?: 'user' | 'admin';
    onLogout?: () => void;
    className?: string;
}

export function AuthStatus({
    variant = 'user',
    onLogout,
    className = '',
}: AuthStatusProps) {
    const [showModal, setShowModal] = useState(false);
    const [showDropdown, setShowDropdown] = useState(false);

    const router = useRouter();
    const { address, isConnected } = useAccount();
    const { disconnect } = useDisconnect();
    const { isAuthenticated, user, logout } = useSharedAuth();

    const handleConnect = useCallback(() => {
        setShowModal(true);
    }, []);

    const handleDisconnect = useCallback(async () => {
        try {
            await logout();
            disconnect();
            setShowDropdown(false);
            onLogout?.();
        } catch (err) {
            logger.error('Disconnect error:', err);
        }
    }, [logout, disconnect, onLogout]);

    const handleAuthSuccess = useCallback(() => {
        setShowModal(false);
        router.refresh();
    }, [router]);

    // Not connected or not authenticated - show connect button
    if (!isConnected || !isAuthenticated) {
        return (
            <>
                <button
                    className={`auth-status-btn ${className}`}
                    onClick={handleConnect}
                >
                    <span className="auth-wallet-icon">🔗</span>
                    <span>{isConnected ? 'Sign In' : 'Connect Wallet'}</span>
                </button>
                <AuthModal
                    isOpen={showModal}
                    onClose={() => setShowModal(false)}
                    variant={variant}
                    onSuccess={handleAuthSuccess}
                />
            </>
        );
    }

    // Authenticated - show address with dropdown
    return (
        <div className="auth-status" style={{ position: 'relative' }}>
            <button
                className={`auth-status-btn ${className}`}
                onClick={() => setShowDropdown(!showDropdown)}
            >
                <span className="auth-status-dot" />
                <span className="auth-status-address">
                    {address?.slice(0, 6)}...{address?.slice(-4)}
                </span>
                <span style={{ fontSize: '0.75rem' }}>▼</span>
            </button>

            {showDropdown && (
                <UserDropdown
                    address={address}
                    user={user}
                    onDisconnect={() => { void handleDisconnect(); }}
                    onClose={() => setShowDropdown(false)}
                />
            )}
        </div>
    );
}

interface UserDropdownProps {
    address: string | undefined;
    user: UserInfoResponse | null;
    onDisconnect: () => void;
    onClose: () => void;
}

function UserDropdown({ address, user, onDisconnect, onClose }: UserDropdownProps) {
    return (
        <>
            {/* Backdrop to close dropdown */}
            <div
                style={{
                    position: 'fixed',
                    inset: 0,
                    zIndex: 40,
                }}
                onClick={onClose}
            />
            {/* Dropdown menu */}
            <div
                style={{
                    position: 'absolute',
                    top: 'calc(100% + 8px)',
                    right: 0,
                    zIndex: 50,
                    minWidth: '200px',
                    padding: '0.5rem',
                    borderRadius: '12px',
                    background: 'rgba(30, 30, 40, 0.95)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    boxShadow: '0 10px 40px rgba(0, 0, 0, 0.4)',
                }}
            >
                {/* User Info */}
                <div
                    style={{
                        padding: '0.75rem 1rem',
                        borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
                    }}
                >
                    <p
                        style={{
                            margin: 0,
                            fontSize: '0.75rem',
                            color: 'rgba(255, 255, 255, 0.5)',
                        }}
                    >
                        Connected as
                    </p>
                    <p
                        style={{
                            margin: '0.25rem 0 0',
                            fontFamily: 'monospace',
                            fontSize: '0.875rem',
                            color: '#f59e0b',
                        }}
                    >
                        {address?.slice(0, 8)}...{address?.slice(-6)}
                    </p>
                    {user?.tier_level !== undefined && user.tier_level !== '' && (
                        <p
                            style={{
                                margin: '0.5rem 0 0',
                                fontSize: '0.75rem',
                                color: 'rgba(255, 255, 255, 0.6)',
                            }}
                        >
                            Tier: <strong>{user.tier_level}</strong>
                        </p>
                    )}
                </div>

                {/* Disconnect Button */}
                <button
                    onClick={onDisconnect}
                    style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                        width: '100%',
                        margin: '0.5rem 0 0',
                        padding: '0.75rem 1rem',
                        border: 'none',
                        borderRadius: '8px',
                        background: 'rgba(239, 68, 68, 0.1)',
                        color: '#ef4444',
                        fontSize: '0.875rem',
                        cursor: 'pointer',
                        transition: 'all 0.2s ease',
                    }}
                    onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'rgba(239, 68, 68, 0.2)';
                    }}
                    onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'rgba(239, 68, 68, 0.1)';
                    }}
                >
                    <span>🔌</span>
                    <span>Disconnect</span>
                </button>
            </div>
        </>
    );
}

export default AuthStatus;
