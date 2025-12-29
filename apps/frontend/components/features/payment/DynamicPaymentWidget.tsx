/**
 * Dynamic Payment Widget (V2)
 *
 * A unified payment widget that:
 * - Fetches payment context from backend based on URL parameters
 * - Displays appropriate payment form based on context type
 * - Handles context-specific validation and submission
 * - Supports plan, group, product, campaign, and custom contexts
 */

'use client';

import { useCallback, useEffect, useState } from 'react';
import { keccak256, parseUnits, toBytes } from 'viem';
import { useAccount, useChainId, useWaitForTransactionReceipt, useWriteContract } from 'wagmi';

import {
    ContextType,
    PAYMENT_ESCROW_ABI,
    ZERO_LINK_HASH,
} from '@/lib/contracts/PaymentEscrowABI';
import {
    getExplorerTxUrl,
    getPaymentEscrowAddress,
    getTokenAddress
} from '@/lib/contracts/addresses';
import { cn } from '@/lib/utils';

// Supported token type
interface SupportedToken {
    symbol: 'USDT' | 'USDC';
    name: string;
    address: string;
    decimals: number;
}

// Get supported tokens for a chain
function getSupportedTokens(chainId: number): SupportedToken[] {
    const tokens: SupportedToken[] = [];
    try {
        tokens.push({
            symbol: 'USDT',
            name: 'Tether USD',
            address: getTokenAddress('USDT', chainId),
            decimals: 18,
        });
    } catch { /* Token not available on this chain */ }
    try {
        tokens.push({
            symbol: 'USDC',
            name: 'USD Coin',
            address: getTokenAddress('USDC', chainId),
            decimals: 18,
        });
    } catch { /* Token not available on this chain */ }
    return tokens;
}

// Types
interface PaymentContext {
    planId: string | null;
    groupId: string | null;
    linkSlug: string | null;
}

interface PaymentLinkData {
    id: string;
    context_type: 'plan' | 'group' | 'product' | 'campaign' | 'custom';
    context_id?: string;
    slug: string;
    name: string;
    description?: string;
    amount: number;
    currency: string;
    expires_at?: string;
    max_uses?: number;
    current_uses: number;
    is_active: boolean;
    is_usable: boolean;
    link_hash: string;
}

interface DynamicPaymentWidgetProps {
    context: PaymentContext;
    className?: string;
    onPaymentSuccess?: (txHash: string) => void;
    onPaymentError?: (error: Error) => void;
}

// Map context type string to enum value
const contextTypeToEnum: Record<string, ContextType> = {
    plan: ContextType.PLAN,
    group: ContextType.GROUP,
    product: ContextType.PRODUCT,
    campaign: ContextType.CAMPAIGN,
    custom: ContextType.CUSTOM,
};

export function DynamicPaymentWidget({
    context,
    className,
    onPaymentSuccess,
    onPaymentError,
}: DynamicPaymentWidgetProps) {
    const { address, isConnected } = useAccount();
    const chainId = useChainId();

    // State
    const [paymentData, setPaymentData] = useState<PaymentLinkData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedToken, setSelectedToken] = useState<SupportedToken | null>(null);
    const [isPaying, setIsPaying] = useState(false);

    // Contract write hook
    const { writeContract, data: txHash, error: writeError } = useWriteContract();
    const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({
        hash: txHash,
    });

    // Get supported tokens for current chain
    const supportedTokens = getSupportedTokens(chainId);

    // Fetch payment context data
    const fetchPaymentData = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            let data: PaymentLinkData | null = null;

            if (context.linkSlug) {
                // Fetch by dynamic link slug
                const response = await fetch(`/api/public/payment-links/${context.linkSlug}`);
                if (!response.ok) {
                    if (response.status === 404) {
                        throw new Error('Payment link not found');
                    }
                    if (response.status === 410) {
                        throw new Error('Payment link has expired or reached max uses');
                    }
                    throw new Error('Failed to fetch payment link');
                }
                data = await response.json();
            } else if (context.planId) {
                // Fetch plan details
                const response = await fetch(`/api/public/plans/${context.planId}`);
                if (!response.ok) {
                    throw new Error('Plan not found');
                }
                const plan = await response.json();
                data = {
                    id: plan.id,
                    context_type: 'plan',
                    context_id: plan.id,
                    slug: `plan-${plan.id}`,
                    name: plan.name,
                    description: plan.description,
                    amount: plan.price,
                    currency: 'USDT',
                    is_active: true,
                    is_usable: true,
                    current_uses: 0,
                    link_hash: ZERO_LINK_HASH,
                };
            } else if (context.groupId) {
                // Fetch group details  
                const response = await fetch(`/api/permissions/groups/${context.groupId}`);
                if (!response.ok) {
                    throw new Error('Group not found');
                }
                const group = await response.json();
                data = {
                    id: group.id,
                    context_type: 'group',
                    context_id: group.id,
                    slug: `group-${group.id}`,
                    name: group.name,
                    description: group.description,
                    amount: group.price || 0,
                    currency: 'USDT',
                    is_active: true,
                    is_usable: true,
                    current_uses: 0,
                    link_hash: ZERO_LINK_HASH,
                };
            }

            setPaymentData(data);

            // Set default token
            if (supportedTokens.length > 0 && !selectedToken) {
                const defaultToken = supportedTokens.find(t => t.symbol === data?.currency) || supportedTokens[0];
                setSelectedToken(defaultToken);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load payment details');
        } finally {
            setLoading(false);
        }
    }, [context, supportedTokens, selectedToken]);

    useEffect(() => {
        if (context.linkSlug || context.planId || context.groupId) {
            fetchPaymentData();
        } else {
            setLoading(false);
            setError('No payment context provided');
        }
    }, [context, fetchPaymentData]);

    // Handle payment success
    useEffect(() => {
        if (isSuccess && txHash) {
            setIsPaying(false);
            onPaymentSuccess?.(txHash);
        }
    }, [isSuccess, txHash, onPaymentSuccess]);

    // Handle payment error
    useEffect(() => {
        if (writeError) {
            setIsPaying(false);
            onPaymentError?.(writeError);
        }
    }, [writeError, onPaymentError]);

    // Execute payment
    const handlePayment = async () => {
        if (!paymentData || !selectedToken || !address) return;

        setIsPaying(true);
        setError(null);

        try {
            let escrowAddress: string;
            try {
                escrowAddress = getPaymentEscrowAddress(chainId);
            } catch {
                throw new Error('Contract not deployed on this network');
            }

            // Convert context type to enum
            const contextType = contextTypeToEnum[paymentData.context_type] ?? ContextType.CUSTOM;

            // Convert context_id to uint256 (use 0 if not available)
            const contextId = paymentData.context_id
                ? BigInt('0x' + paymentData.context_id.replace(/-/g, ''))
                : BigInt(0);

            // Compute link hash
            const linkHash = paymentData.link_hash !== ZERO_LINK_HASH
                ? paymentData.link_hash as `0x${string}`
                : keccak256(toBytes(paymentData.slug)) as `0x${string}`;

            // Parse amount with token decimals
            const amount = parseUnits(paymentData.amount.toString(), selectedToken.decimals);

            // Call payWithContext on the smart contract
            writeContract({
                address: escrowAddress as `0x${string}`,
                abi: PAYMENT_ESCROW_ABI,
                functionName: 'payWithContext',
                args: [contextType, contextId, linkHash, selectedToken.address as `0x${string}`, amount],
            });
        } catch (err) {
            setIsPaying(false);
            setError(err instanceof Error ? err.message : 'Payment failed');
            onPaymentError?.(err instanceof Error ? err : new Error('Payment failed'));
        }
    };

    // Loading state
    if (loading) {
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="flex flex-col items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
                    <p className="mt-4 text-gray-600 dark:text-gray-400">Loading payment details...</p>
                </div>
            </div>
        );
    }

    // Error state
    if (error || !paymentData) {
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="text-center py-12">
                    <div className="text-6xl mb-4">❌</div>
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                        Payment Unavailable
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                        {error || 'This payment link is no longer available'}
                    </p>
                </div>
            </div>
        );
    }

    // Not usable state
    if (!paymentData.is_usable) {
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="text-center py-12">
                    <div className="text-6xl mb-4">⏰</div>
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                        Link Expired
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                        This payment link has expired or reached its maximum uses.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
            {/* Header */}
            <div className="text-center mb-8">
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-purple-500 to-pink-500 text-white text-2xl mb-4">
                    💳
                </div>
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                    {paymentData.name}
                </h2>
                {paymentData.description && (
                    <p className="text-gray-600 dark:text-gray-400">{paymentData.description}</p>
                )}
                <div className="mt-4 inline-flex items-center px-3 py-1 rounded-full bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 text-sm capitalize">
                    {paymentData.context_type} Payment
                </div>
            </div>

            {/* Amount Display */}
            <div className="bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 rounded-xl p-6 mb-6">
                <div className="text-center">
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">Amount to Pay</p>
                    <p className="text-4xl font-black bg-gradient-to-r from-purple-600 to-pink-600 bg-clip-text text-transparent">
                        {paymentData.amount} {paymentData.currency}
                    </p>
                </div>
            </div>

            {/* Token Selection */}
            {supportedTokens.length > 1 && (
                <div className="mb-6">
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Pay with
                    </label>
                    <div className="grid grid-cols-3 gap-2">
                        {supportedTokens.map((token) => (
                            <button
                                key={token.symbol}
                                onClick={() => setSelectedToken(token)}
                                className={cn(
                                    'px-4 py-3 rounded-lg border-2 transition-all',
                                    selectedToken?.symbol === token.symbol
                                        ? 'border-purple-500 bg-purple-50 dark:bg-purple-900/30'
                                        : 'border-gray-200 dark:border-gray-700 hover:border-purple-300'
                                )}
                            >
                                <span className="font-medium">{token.symbol}</span>
                            </button>
                        ))}
                    </div>
                </div>
            )}

            {/* Expiration Info */}
            {paymentData.expires_at && (
                <div className="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg border border-yellow-200 dark:border-yellow-800">
                    <p className="text-sm text-yellow-800 dark:text-yellow-200">
                        ⏰ This link expires on {new Date(paymentData.expires_at).toLocaleString()}
                    </p>
                </div>
            )}

            {/* Usage Info */}
            {paymentData.max_uses && (
                <div className="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
                    <p className="text-sm text-blue-800 dark:text-blue-200">
                        📊 {paymentData.max_uses - paymentData.current_uses} uses remaining
                    </p>
                </div>
            )}

            {/* Pay Button */}
            {!isConnected ? (
                <div className="text-center py-4">
                    <p className="text-gray-600 dark:text-gray-400 mb-4">
                        Connect your wallet to continue
                    </p>
                    {/* Wallet connect button will be provided by parent component */}
                </div>
            ) : (
                <button
                    onClick={handlePayment}
                    disabled={isPaying || isConfirming || !selectedToken}
                    className={cn(
                        'w-full py-4 rounded-xl font-bold text-lg transition-all',
                        isPaying || isConfirming
                            ? 'bg-gray-400 cursor-not-allowed'
                            : 'bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white shadow-lg hover:shadow-xl'
                    )}
                >
                    {isPaying || isConfirming ? (
                        <span className="flex items-center justify-center">
                            <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white mr-2"></div>
                            {isConfirming ? 'Confirming...' : 'Processing...'}
                        </span>
                    ) : (
                        `Pay ${paymentData.amount} ${selectedToken?.symbol || paymentData.currency}`
                    )}
                </button>
            )}

            {/* Transaction Success */}
            {isSuccess && txHash && (
                <div className="mt-6 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
                    <div className="flex items-center">
                        <span className="text-2xl mr-3">✅</span>
                        <div>
                            <p className="font-medium text-green-800 dark:text-green-200">Payment Successful!</p>
                            <a
                                href={getExplorerTxUrl(chainId, txHash)}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-sm text-green-600 dark:text-green-400 hover:underline"
                            >
                                View on Explorer →
                            </a>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
