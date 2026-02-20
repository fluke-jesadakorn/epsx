 
'use client';

/**
 * Chain Verification Card
 * 
 * Displays when user is connected to an unsupported blockchain network.
 * Provides options to switch to a supported network or add it to MetaMask.
 */

import { AlertTriangle, ArrowRight, CheckCircle2, ExternalLink, Loader2, Plus, Wifi } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useAccount, useChainId, useSwitchChain } from 'wagmi';

import { CHAIN_EXPLORERS, isPaymentEscrowDeployed, PAYMENT_ESCROW_ADDRESS } from '@/lib/contracts/addresses';
import { cn } from '@/lib/utils';
import type { ChainInfo} from '@/shared/components/navigation/chain-selector';
import { supportedChains } from '@/shared/components/navigation/chain-selector';

/**
 * Network configuration for wallet_addEthereumChain RPC
 */
interface NetworkConfig {
    chainId: string;
    chainName: string;
    nativeCurrency: { name: string; symbol: string; decimals: number };
    rpcUrls: string[];
    blockExplorerUrls?: string[];
}

/**
 * Error with code property from wallet/RPC
 */
interface WalletError extends Error {
    code?: number;
    [key: string]: unknown;
}

/**
 * Ethereum provider interface for wallet_addEthereumChain
 */
interface EthereumProvider {
    request(args: { method: string; params: unknown[] }): Promise<unknown>;
    [key: string]: unknown;
}

/**
 * Get the local Anvil RPC URL based on current browser hostname.
 * Supports Tailscale IPs (100.x.x.x) and other local network IPs.
 */
function getLocalAnvilRpcUrl(): string {
    if (typeof window === 'undefined') {
        return 'http://127.0.0.1:8545';
    }
    const hostname = window.location.hostname;
    // Use the same host for Anvil RPC on port 8545
    if (hostname === 'localhost' || hostname === '127.0.0.1') {
        return 'http://127.0.0.1:8545';
    }
    // For Tailscale IPs (100.x.x.x) or other local IPs, use the same host
    return `http://${hostname}:8545`;
}

/**
 * Get network configs dynamically (local Anvil uses current hostname)
 */
function getNetworkConfigs(): Record<number, NetworkConfig> {
    return {
        56: {
            chainId: '0x38',
            chainName: 'BNB Smart Chain',
            nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 },
            rpcUrls: ['https://bsc-dataseed.binance.org/'],
            blockExplorerUrls: ['https://bscscan.com'],
        },
        97: {
            chainId: '0x61',
            chainName: 'BSC Testnet',
            nativeCurrency: { name: 'tBNB', symbol: 'tBNB', decimals: 18 },
            rpcUrls: ['https://data-seed-prebsc-1-s1.binance.org:8545/'],
            blockExplorerUrls: ['https://testnet.bscscan.com'],
        },
        31337: {
            chainId: '0x7a69',
            chainName: 'Anvil Local (BSC Fork)',
            nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 },
            rpcUrls: [getLocalAnvilRpcUrl()],
        },
    };
}

interface ChainVerificationCardProps {
    /** Optional callback when chain switches to a supported network */
    onChainReady?: () => void;
    /** Additional CSS classes */
    className?: string;
}

/**
 * Truncate address for display: 0x1234...abcd
 */
function truncateAddress(address: string): string {
    if (!address || address.length < 12) {return address;}
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
 
export function ChainVerificationCard({
    onChainReady,
    className,
}: ChainVerificationCardProps) {
    const chainId = useChainId();
    const { isConnected, connector } = useAccount();
    const { switchChain, isPending: isSwitching, error: switchError } = useSwitchChain();
    const [selectedChain, setSelectedChain] = useState<ChainInfo | null>(null);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const [isAddingNetwork, setIsAddingNetwork] = useState(false);

    // Filter chains to only show ones with deployed contracts
     
    const deployedChains = useMemo(
        () => supportedChains.filter((chain) => {
             
            const deployed = isPaymentEscrowDeployed(chain.id);
            return typeof deployed === 'boolean' ? deployed : Boolean(deployed);
        }),
        []
    );

    // Check if current chain is supported AND has contract deployed
     
    const isSupported = useMemo(
        () => {
            const inSupportedList = supportedChains.some((chain) => chain.id === chainId);
             
            const hasContract = isPaymentEscrowDeployed(chainId);
            const isDeployed = typeof hasContract === 'boolean' ? hasContract : Boolean(hasContract);
            return inSupportedList && isDeployed;
        },
        [chainId]
    );
    const currentChain = supportedChains.find(chain => chain.id === chainId);

    // Notify parent when chain becomes supported
    useEffect(() => {
        if (isSupported && onChainReady) {
            onChainReady();
        }
    }, [isSupported, onChainReady]);

    // Handle switch errors
    useEffect(() => {
        if (switchError) {
            const error = switchError as WalletError;
            if (error?.code === 4902) {
                setErrorMessage('Network not found in wallet. Click "Add Network" to add it first.');
            } else if (error?.code === -32002) {
                setErrorMessage('Please check your wallet for pending requests.');
            } else if (error?.code === 4001) {
                setErrorMessage('Network switch cancelled.');
            } else {
                setErrorMessage('Failed to switch network. Please try again.');
            }
        }
    }, [switchError]);

    /**
     * Add network to MetaMask using wallet_addEthereumChain RPC
     */
    const handleAddNetwork = async (chain: ChainInfo) => {
        if (!connector || isAddingNetwork) {return;}

        const config = getNetworkConfigs()[chain.id];
        if (!config) {
            setErrorMessage('Network configuration not available.');
            return;
        }

        setIsAddingNetwork(true);
        setSelectedChain(chain);
        setErrorMessage(null);

        try {
            const provider = await connector.getProvider() as EthereumProvider;
            await provider.request({
                method: 'wallet_addEthereumChain',
                params: [config],
            });
            // After adding, try to switch to it
             
            switchChain({ chainId: chain.id });
        } catch (err: unknown) {
            const error = err as WalletError;
            if (error?.code === 4001) {
                setErrorMessage('User rejected adding the network.');
            } else {
                setErrorMessage('Failed to add network. Please try again.');
            }
        } finally {
            setIsAddingNetwork(false);
        }
    };

    const handleSwitchChain = async (chain: ChainInfo) => {
        if (!isConnected || isSwitching) {return;}

        setSelectedChain(chain);
        setErrorMessage(null);

        try {
             
            switchChain({ chainId: chain.id });
        } catch (err: unknown) {
            // If network not found, suggest adding it
            const error = err as WalletError;
            if (error?.code === 4902) {
                setErrorMessage('Network not found. Click "Add Network" button below.');
            }
        }
    };

    /**
     * Get contract info for a chain
     */
    const getContractInfo = (chainIdNum: number) => {
        const hasContract = isPaymentEscrowDeployed(chainIdNum);
        const address = hasContract ? PAYMENT_ESCROW_ADDRESS[chainIdNum] : null;
        const explorer = CHAIN_EXPLORERS[chainIdNum as keyof typeof CHAIN_EXPLORERS];
        return { hasContract, address, explorer };
    };

    // If already on supported chain, show success state briefly
    if (isSupported && currentChain) {
        const contractInfo = getContractInfo(currentChain.id);
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="text-center py-8">
                    <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-r from-green-400 to-emerald-500 text-white mb-6">
                        <CheckCircle2 className="w-10 h-10" />
                    </div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                        Network Verified
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-4">
                        Connected to {currentChain.displayName}
                    </p>
                    {/* Contract Info */}
                    {contractInfo.address && (
                        <div className="inline-flex items-center gap-2 px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg">
                            <span className="text-xs text-gray-500 dark:text-gray-400">Contract:</span>
                            <code className="text-xs font-mono text-gray-700 dark:text-gray-300">
                                {truncateAddress(contractInfo.address)}
                            </code>
                            {contractInfo.explorer && currentChain.id !== 31337 && (
                                <a
                                    href={contractInfo.explorer.address(contractInfo.address)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-purple-500 hover:text-purple-600"
                                >
                                    <ExternalLink className="w-3 h-3" />
                                </a>
                            )}
                        </div>
                    )}
                </div>
            </div>
        );
    }

    return (
        <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
            {/* Header */}
            <div className="text-center mb-8">
                <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-r from-amber-400 to-orange-500 text-white mb-6 animate-pulse">
                    <AlertTriangle className="w-10 h-10" />
                </div>
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                    Wrong Network
                </h2>
                <p className="text-gray-600 dark:text-gray-400 max-w-md mx-auto">
                    Please switch to a supported network to continue with your payment.
                </p>
            </div>

            {/* Network Selection */}
            <div className="space-y-4 mb-6">
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                    Available Networks
                </label>
                {deployedChains.map((chain) => {
                    const contractInfo = getContractInfo(chain.id);
                    const isSelected = selectedChain?.id === chain.id;
                    const isLoading = isSelected && (isSwitching || isAddingNetwork);

                    return (
                        <div
                            key={chain.id}
                            className={cn(
                                'rounded-xl border-2 transition-all overflow-hidden',
                                isLoading
                                    ? 'border-purple-500 bg-purple-50 dark:bg-purple-900/30'
                                    : 'border-gray-200 dark:border-gray-700 hover:border-purple-300'
                            )}
                        >
                            {/* Main Chain Row */}
                            <button
                                onClick={() => handleSwitchChain(chain)}
                                disabled={isSwitching || isAddingNetwork}
                                className={cn(
                                    'w-full flex items-center gap-4 p-4 transition-all',
                                    'hover:bg-purple-50 dark:hover:bg-purple-900/20',
                                    (isSwitching || isAddingNetwork) && !isSelected && 'opacity-50 cursor-not-allowed'
                                )}
                            >
                                {/* Chain Icon */}
                                <div className={cn(
                                    'w-12 h-12 rounded-full flex items-center justify-center text-2xl shadow-md flex-shrink-0',
                                    chain.testnet
                                        ? chain.id === 31337
                                            ? 'bg-gradient-to-br from-slate-400 to-slate-600'
                                            : 'bg-gradient-to-br from-purple-400 to-purple-600'
                                        : 'bg-gradient-to-br from-yellow-400 to-yellow-600'
                                )}>
                                    {chain.icon}
                                </div>

                                {/* Chain Info */}
                                <div className="flex-1 text-left min-w-0">
                                    <div className="font-semibold text-gray-900 dark:text-white">
                                        {chain.displayName}
                                    </div>
                                    <div className="text-sm text-gray-500 dark:text-gray-400">
                                        {chain.testnet ? 'Testnet' : 'Mainnet'} • Chain ID: {chain.id}
                                    </div>
                                </div>

                                {/* Action Indicator */}
                                {isLoading ? (
                                    <Loader2 className="w-6 h-6 text-purple-500 animate-spin flex-shrink-0" />
                                ) : (
                                    <ArrowRight className="w-6 h-6 text-gray-400 flex-shrink-0" />
                                )}
                            </button>

                            {/* Contract & Add Network Info */}
                            <div className="px-4 pb-4 pt-0 flex items-center justify-between gap-2 flex-wrap">
                                {/* Contract Info */}
                                {contractInfo.hasContract && contractInfo.address && (
                                    <div className="flex items-center gap-2">
                                        <span className="w-2 h-2 rounded-full bg-green-500" />
                                        <span className="text-xs text-gray-500 dark:text-gray-400">Contract:</span>
                                        <code className="text-xs font-mono text-gray-600 dark:text-gray-300">
                                            {truncateAddress(contractInfo.address)}
                                        </code>
                                        {contractInfo.explorer && chain.id !== 31337 && (
                                            <a
                                                href={contractInfo.explorer.address(contractInfo.address)}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-purple-500 hover:text-purple-600"
                                                onClick={(e) => e.stopPropagation()}
                                            >
                                                <ExternalLink className="w-3 h-3" />
                                            </a>
                                        )}
                                    </div>
                                )}

                                {/* Add Network Button */}
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleAddNetwork(chain);
                                    }}
                                    disabled={isAddingNetwork || isSwitching}
                                    className={cn(
                                        'inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all',
                                        'bg-purple-100 text-purple-700 hover:bg-purple-200',
                                        'dark:bg-purple-900/40 dark:text-purple-300 dark:hover:bg-purple-900/60',
                                        (isAddingNetwork || isSwitching) && 'opacity-50 cursor-not-allowed'
                                    )}
                                >
                                    {isSelected && isAddingNetwork ? (
                                        <Loader2 className="w-3 h-3 animate-spin" />
                                    ) : (
                                        <Plus className="w-3 h-3" />
                                    )}
                                    Add to Wallet
                                </button>
                            </div>
                        </div>
                    );
                })}
            </div>

            {/* Error Message */}
            {errorMessage && (
                <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
                    <p className="text-sm text-red-700 dark:text-red-300 flex items-center gap-2">
                        <AlertTriangle className="w-4 h-4" />
                        {errorMessage}
                    </p>
                </div>
            )}

            {/* Current Network Info */}
            <div className="p-4 bg-gray-50 dark:bg-card rounded-xl border border-gray-200 dark:border-gray-700">
                <div className="flex items-center gap-3">
                    <Wifi className="w-5 h-5 text-gray-400" />
                    <div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                            Current Network
                        </div>
                        <div className="text-sm font-medium text-gray-700 dark:text-gray-300">
                            Chain ID: {chainId} (Unsupported)
                        </div>
                    </div>
                </div>
            </div>

            {/* Help Text */}
            <p className="mt-6 text-center text-sm text-gray-500 dark:text-gray-400">
                Click on a network to switch, or use "Add to Wallet" to add the network first.
            </p>
        </div>
    );
}
