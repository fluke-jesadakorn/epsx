'use client';

import { Link, Wifi } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useAccount, useChainId, useSwitchChain } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';
import { isProduction } from '../../utils';
import {
    Button,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '../ui';

interface ChainSelectorProps {
    className?: string;
    compact?: boolean;
}

export interface ChainInfo {
    id: number;
    name: string;
    displayName: string;
    icon: string;
    testnet: boolean;
}

// All chains configuration - Anvil is filtered out in production
const allChains: ChainInfo[] = [
    {
        id: bsc.id,
        name: bsc.name,
        displayName: 'BSC Mainnet',
        icon: '🟡',
        testnet: false,
    },
    {
        id: bscTestnet.id,
        name: bscTestnet.name,
        displayName: 'BSC Testnet',
        icon: '🟠',
        testnet: true,
    },
    {
        id: 31337,
        name: 'Anvil Local (BSC Fork)',
        displayName: 'Anvil Local (BSC)',
        icon: '🔧',
        testnet: true,
    },
];

// Export filtered chains - Anvil (31337) only available in local development
export const supportedChains: ChainInfo[] = isProduction
    ? allChains.filter(chain => chain.id !== 31337)
    : allChains.filter(chain => {
        // Only show Anvil if on localhost
        if (chain.id === 31337) {
            return typeof window !== 'undefined' &&
                (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1');
        }
        return true;
    });

export function ChainSelector({ className = '', compact = false }: ChainSelectorProps) {
    const [isHydrated, setIsHydrated] = useState(false);
    const [isOpen, setIsOpen] = useState(false);
    const chainId = useChainId();
    const { switchChainAsync, isPending: isSwitching } = useSwitchChain();
    const { isConnected } = useAccount();

    useEffect(() => {
        setIsHydrated(true);
    }, []);

    // Hide chain selector in production - keep code for future multi-chain support
    if (isProduction) {
        return null;
    }

    if (!isHydrated) {
        return (
            <Button
                variant="ghost"
                size="icon"
                className={`h-10 w-10 rounded-full bg-orange-50 text-orange-500 hover:bg-orange-100 dark:bg-slate-800 dark:text-orange-400 dark:hover:bg-slate-700 ${className}`}
                disabled
            >
                <Wifi className="h-4 w-4 text-orange-500" />
            </Button>
        );
    }

    const currentChain = supportedChains.find(chain => chain.id === chainId);
    const isSupported = Boolean(currentChain);

    const handleChainSwitch = async (targetChainId: number) => {
        if (!isConnected || isSwitching || targetChainId === chainId) { return; }

        try {
            await switchChainAsync({ chainId: targetChainId });
            setIsOpen(false);
        } catch (_err: unknown) {
            // Keep dropdown open on error
        }
    };

    return (
        <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    size={compact ? "sm" : "default"}
                    disabled={isSwitching || !isConnected}
                    className={`
            ${compact ? 'h-8 px-3 text-xs' : 'h-10 px-4 text-sm'}
            rounded-full bg-transparent hover:bg-slate-50/80 hover:text-slate-700
            dark:bg-transparent dark:hover:bg-white/10 dark:hover:text-slate-200
            disabled:opacity-50 disabled:cursor-not-allowed
            ${className}
          `}
                >
                    <TriggerContent compact={compact} currentChain={currentChain} />
                </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent
                align="end"
                style={{ zIndex: 99999 }}
                className="w-56 p-2 bg-white border border-slate-200 shadow-xl dark:bg-slate-900 dark:border-slate-700"
            >
                <div className="px-2 py-1.5 mb-1">
                    <span className="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                        Select Network
                    </span>
                </div>

                {supportedChains.map((chain) => (
                    <ChainMenuItem
                        key={chain.id}
                        chain={chain}
                        currentChainId={chainId}
                        isSwitching={isSwitching}
                        onSwitch={(id) => { void handleChainSwitch(id); }}
                    />
                ))}

                {!isConnected && (
                    <div className="mt-2 border-t border-slate-200 dark:border-slate-700 pt-2 px-3 pb-1 text-xs text-slate-500 dark:text-slate-400 italic">
                        Connect your wallet to switch networks
                    </div>
                )}

                {isConnected && !isSupported && (
                    <div className="mt-2 border-t border-slate-200 dark:border-slate-700 pt-2 px-3 pb-1 text-xs text-red-600 dark:text-red-400">
                        Current network not supported
                    </div>
                )}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

function TriggerContent({ compact, currentChain }: { compact: boolean; currentChain?: ChainInfo }) {
    return (
        <div className="flex items-center gap-2">
            <Link className="h-4 w-4 text-orange-500" />
            <span className={compact ? "text-xs font-medium" : "text-sm font-medium"}>
                {currentChain?.displayName ?? 'Unknown'}
            </span>
        </div>
    );
}

function ChainMenuItem({
    chain,
    currentChainId,
    isSwitching,
    onSwitch
}: {
    chain: ChainInfo;
    currentChainId: number;
    isSwitching: boolean;
    onSwitch: (id: number) => void;
}) {
    return (
        <DropdownMenuItem
            onClick={() => { onSwitch(chain.id); }}
            disabled={isSwitching || chain.id === currentChainId}
            className={`
        flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-colors
        ${chain.id === currentChainId
                    ? 'bg-orange-50 text-slate-900 dark:bg-slate-800 dark:text-orange-300'
                    : 'text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-white/10'
                }
        disabled:opacity-50 disabled:cursor-not-allowed
      `}
        >
            <div className={`w-6 h-6 rounded-full flex items-center justify-center shadow-sm ${chain.testnet
                ? chain.id === 31337
                    ? 'bg-gradient-to-br from-slate-400 to-slate-600'
                    : 'bg-gradient-to-br from-purple-400 to-purple-600'
                : 'bg-gradient-to-br from-yellow-400 to-yellow-600'
                }`}>
                <span className="text-xs text-white">{chain.icon}</span>
            </div>
            <div className="flex-1">
                <div className="text-sm font-medium">{chain.displayName}</div>
                <div className={`text-xs ${chain.id === currentChainId ? 'text-slate-600 dark:text-orange-400/80' : 'text-slate-500 dark:text-slate-400'}`}>
                    {chain.testnet ? 'Testnet' : 'Mainnet'}
                </div>
            </div>
            {chain.id === currentChainId && (
                <div className="w-5 h-5 rounded-full bg-green-500 flex items-center justify-center">
                    <span className="text-white text-xs">✓</span>
                </div>
            )}
        </DropdownMenuItem>
    )
}
