'use client';

import { Check, ChevronRight, Copy, ExternalLink, LogOut, Wallet } from 'lucide-react';
import Link from 'next/link';
import { useState } from 'react';
import { formatAddress } from '../../auth/utils';
import { getExplorerAddressLink } from '../../config/constants';
import { logger } from '../../utils/logger';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { useSharedAuth } from './provider';

interface WalletProviderInfo {
    name: string;
    icon: string;
    color: string;
}

const walletProviders: Record<string, WalletProviderInfo> = {
    metaMask: { name: 'MetaMask', icon: '🦊', color: 'bg-orange-500' },
    walletConnect: { name: 'WalletConnect', icon: '🔗', color: 'bg-blue-500' },
    injected: { name: 'Browser Wallet', icon: '🌐', color: 'bg-purple-500' },
    coinbase: { name: 'Coinbase', icon: '🔵', color: 'bg-blue-600' },
    rainbow: { name: 'Rainbow', icon: '🌈', color: 'bg-gradient-to-r from-pink-500 to-violet-500' },
};

export interface ConnectedWalletDisplayProps {
    address: string;
    connectorId?: string;
    isAuthenticated: boolean;
    isAuthenticating?: boolean;
    showDisconnect?: boolean;
    onSignIn?: () => void;
    onDisconnect?: () => Promise<void> | void;
    customActions?: React.ReactNode;
    navigationLinks?: Array<{ href: string; label: string; icon: React.ComponentType<{ className?: string }> }>;
    className?: string;
    compact?: boolean;
}

export function ConnectedWalletDisplay({
    address,
    connectorId = 'injected',
    isAuthenticated,
    isAuthenticating = false,
    showDisconnect = true,
    onSignIn,
    onDisconnect,
    customActions,
    navigationLinks = [],
    className = '',
    compact = false,
}: ConnectedWalletDisplayProps) {
    const [isOpen, setIsOpen] = useState(false);
    const { logout } = useSharedAuth();

    const status = getStatusInfo(isAuthenticated, isAuthenticating);

    const handleDisconnectAction = async () => {
        await processDisconnect(onDisconnect, logout);
        setIsOpen(false);
    };

    return (
        <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
            <DropdownMenuTrigger asChild>
                <button
                    className={`flex items-center gap-2 rounded-2xl px-3 py-2 text-sm font-medium transition-all duration-200
                        text-slate-600 hover:bg-slate-50/80 hover:text-slate-700
                        dark:text-slate-300 dark:hover:bg-white/10 dark:bg-slate-800/40 dark:hover:text-slate-200
                        bg-transparent border-0 ${className}`}
                >
                    <Wallet className={`h-4 w-4 ${isAuthenticated ? 'text-emerald-500' : 'text-orange-500'}`} />
                    {!compact && <span className="text-sm font-medium">{formatAddress(address)}</span>}
                </button>
            </DropdownMenuTrigger>

            <WalletDropdownMenuContent
                address={address}
                connectorId={connectorId}
                status={status}
                isAuthenticated={isAuthenticated}
                isAuthenticating={isAuthenticating}
                showDisconnect={showDisconnect}
                onSignIn={onSignIn}
                handleDisconnect={() => { void handleDisconnectAction(); }}
                customActions={customActions}
                navigationLinks={navigationLinks}
            />
        </DropdownMenu>
    );
}

interface WalletDropdownMenuContentProps {
    address: string;
    connectorId: string;
    status: { text: string; color: string };
    isAuthenticated: boolean;
    isAuthenticating: boolean;
    showDisconnect: boolean;
    onSignIn?: () => void;
    handleDisconnect: () => void;
    customActions?: React.ReactNode;
    navigationLinks: Array<{ href: string; label: string; icon: React.ComponentType<{ className?: string }> }>;
}

function WalletDropdownMenuContent({
    address,
    connectorId,
    status,
    isAuthenticated,
    isAuthenticating,
    showDisconnect,
    onSignIn,
    handleDisconnect,
    customActions,
    navigationLinks
}: WalletDropdownMenuContentProps) {
    const [copied, setCopied] = useState(false);
    const providerInfo = walletProviders[connectorId.toLowerCase()] ?? walletProviders.injected;

    const handleCopyAddress = async () => {
        if (!address) { return; }
        try {
            await navigator.clipboard.writeText(address);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (e) {
            logger.error('Failed to copy address:', e);
        }
    };

    const handleViewExplorer = () => {
        const explorerUrl = getExplorerAddressLink(address);
        window.open(explorerUrl, '_blank', 'noopener,noreferrer');
    };

    return (
        <DropdownMenuContent
            align="end"
            sideOffset={8}
            className="w-72 p-0 bg-white/98 backdrop-blur-xl border border-slate-200 dark:bg-slate-900/95 dark:border-slate-700 shadow-2xl rounded-2xl overflow-hidden"
            style={{ zIndex: 99999 }}
        >
            <WalletHeader address={address} providerInfo={providerInfo} status={status} isAuthenticated={isAuthenticated} />
            <QuickActions handleCopyAddress={() => { void handleCopyAddress(); }} handleViewExplorer={handleViewExplorer} copied={copied} />

            <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />

            {!isAuthenticated && !isAuthenticating && onSignIn && (
                <>
                    <div className="p-2">
                        <button
                            onClick={onSignIn}
                            className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
                                bg-gradient-to-r from-emerald-50 to-green-50 hover:from-emerald-100 hover:to-green-100
                                dark:from-emerald-900/30 dark:to-green-900/30 dark:hover:from-emerald-900/50 dark:hover:to-green-900/50
                                border border-emerald-200/50 dark:border-emerald-700/30
                                text-emerald-700 dark:text-emerald-300 transition-all duration-150"
                        >
                            <Wallet className="h-4 w-4" />
                            <div className="flex-1 text-left">
                                <div className="text-sm font-semibold">Sign In with Wallet</div>
                                <div className="text-xs opacity-75">Authenticate to access all features</div>
                            </div>
                        </button>
                    </div>
                    <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />
                </>
            )}

            {customActions !== null && customActions !== undefined && (
                <>
                    <div className="p-2">{customActions}</div>
                    <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />
                </>
            )}

            <NavigationLinks navigationLinks={navigationLinks} />

            {showDisconnect && (
                <div className="p-2">
                    <button
                        onClick={handleDisconnect}
                        className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl
                            bg-red-50 hover:bg-red-100 dark:bg-red-900/20 dark:hover:bg-red-900/30
                            text-red-600 dark:text-red-400 transition-all duration-150"
                    >
                        <LogOut className="h-4 w-4" />
                        <span className="text-sm font-medium">Disconnect Wallet</span>
                    </button>
                </div>
            )}
        </DropdownMenuContent>
    );
}

interface WalletHeaderProps {
    address: string;
    providerInfo: WalletProviderInfo;
    status: { text: string; color: string };
    isAuthenticated: boolean;
}

function WalletHeader({ address, providerInfo, status, isAuthenticated }: WalletHeaderProps) {
    return (
        <div className="p-4 bg-gradient-to-br from-slate-50 to-orange-50/50 dark:from-slate-800 dark:to-orange-900/20 border-b border-slate-200/50 dark:border-slate-700/50">
            <div className="flex items-center gap-3">
                <div className="flex items-center justify-center w-10 h-10 rounded-full bg-white dark:bg-slate-700 shadow-sm">
                    <span className="text-xl">{providerInfo.icon}</span>
                </div>
                <div className="flex-1 min-w-0">
                    <div className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                        {providerInfo.name}
                    </div>
                    <div className="text-xs font-mono text-slate-500 dark:text-slate-400 truncate">
                        {address}
                    </div>
                    <div className={`text-xs font-medium ${status.color} flex items-center gap-1 mt-0.5`}>
                        {isAuthenticated && <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />}
                        {status.text}
                    </div>
                </div>
            </div>
        </div>
    );
}

interface QuickActionsProps {
    handleCopyAddress: () => void;
    handleViewExplorer: () => void;
    copied: boolean;
}

function QuickActions({ handleCopyAddress, handleViewExplorer, copied }: QuickActionsProps) {
    return (
        <div className="p-2 flex gap-2">
            <button
                onClick={handleCopyAddress}
                className="flex-1 flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl
                    bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700
                    text-slate-700 dark:text-slate-300 text-sm font-medium transition-all duration-150"
            >
                {copied ? <Check className="h-4 w-4 text-emerald-500" /> : <Copy className="h-4 w-4" />}
                {copied ? 'Copied!' : 'Copy'}
            </button>
            <button
                onClick={handleViewExplorer}
                className="flex-1 flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl
                    bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700
                    text-slate-700 dark:text-slate-300 text-sm font-medium transition-all duration-150"
            >
                <ExternalLink className="h-4 w-4" />
                Explorer
            </button>
        </div>
    );
}

interface NavigationLinksProps {
    navigationLinks: Array<{ href: string; label: string; icon: React.ComponentType<{ className?: string }> }>;
}

function NavigationLinks({ navigationLinks }: NavigationLinksProps) {
    if (navigationLinks.length === 0) {
        return null;
    }
    return (
        <>
            <div className="p-2 space-y-1">
                {navigationLinks.map((link) => (
                    <DropdownMenuItem key={link.href} asChild className="p-0">
                        <Link
                            href={link.href}
                            className="flex items-center gap-3 px-3 py-2.5 rounded-xl cursor-pointer
                                hover:bg-slate-100 dark:hover:bg-white/10 dark:bg-slate-800 transition-colors"
                        >
                            <link.icon className="h-4 w-4 text-slate-500" />
                            <span className="flex-1 text-sm font-medium text-slate-700 dark:text-slate-300">
                                {link.label}
                            </span>
                            <ChevronRight className="h-4 w-4 text-slate-400" />
                        </Link>
                    </DropdownMenuItem>
                ))}
            </div>
            <DropdownMenuSeparator className="bg-slate-200/50 dark:bg-slate-700/50 my-0" />
        </>
    );
}

/**
 * Internal helper to get status info
 */
function getStatusInfo(isAuthenticated: boolean, isAuthenticating: boolean) {
    if (isAuthenticating) {
        return { text: 'Signing...', color: 'text-orange-500' };
    }
    if (isAuthenticated) {
        return { text: 'Authenticated', color: 'text-emerald-500' };
    }
    return { text: 'Connected', color: 'text-slate-500' };
}

/**
 * Internal helper to handle disconnect logic
 */
async function processDisconnect(
    onDisconnect: ConnectedWalletDisplayProps['onDisconnect'],
    logout: () => Promise<void>
) {
    try {
        if (onDisconnect) {
            const result = onDisconnect();
            if (result instanceof Promise) {
                await result;
            }
        } else {
            await logout();
        }
    } catch (error) {
        logger.error('Logout error:', error);
    }
}

export default ConnectedWalletDisplay;
