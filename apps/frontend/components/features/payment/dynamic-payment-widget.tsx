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

import { AlertTriangle, CheckCircle, Loader2, RefreshCw } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { getAddress, parseUnits } from 'viem';
import { useAccount, useBalance, useChainId, useReadContract } from 'wagmi';

import { createFrontendApiClient } from '@/shared/utils/api-client';

import { ZERO_LINK_HASH } from '@/lib/contracts/payment-escrow-abi';
import {
    getExplorerTxUrl,
    getPaymentReceiverAddress,
    getTokenAddress,
} from '@/lib/contracts/addresses';
import { cn } from '@/lib/utils';
import { supportedChains } from '@/shared/components/navigation/chain-selector';
import { devLog } from '@/shared/utils';
import { ChainVerificationCard } from './chain-verification-card';
import { useAddTokenToWallet } from './hooks/use-add-token-to-wallet';
import { useDirectTokenTransfer } from './hooks/use-direct-token-transfer';
import { usePublicBalance } from './hooks/use-public-balance';

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
    } catch (_err) { /* Token not available on this chain */ }
    try {
        tokens.push({
            symbol: 'USDC',
            name: 'USD Coin',
            address: getTokenAddress('USDC', chainId),
            decimals: 18,
        });
    } catch (_err) { /* Token not available on this chain */ }
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

// eslint-disable-next-line max-lines-per-function,complexity
export function DynamicPaymentWidget({
    context,
    className,
    onPaymentSuccess,
    onPaymentError: _onPaymentError,
}: DynamicPaymentWidgetProps) {
    const { address, isConnected } = useAccount();
    const chainId = useChainId();

    // State
    const [paymentData, setPaymentData] = useState<PaymentLinkData | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedToken, setSelectedToken] = useState<SupportedToken | null>(null);
    const [tokenVerificationStatus, setTokenVerificationStatus] = useState<'pending' | 'checking' | 'verified' | 'failed'>('pending');

    // Memoize supported tokens to prevent recreation on each render
    const supportedTokens = useMemo(() => getSupportedTokens(chainId), [chainId]);

    // Check if current chain is supported
    const isChainSupported = useMemo(
        () => supportedChains.some(chain => chain.id === chainId),
        [chainId]
    );

    // Get receiver address for this chain (direct transfer)
     
    const receiverAddress = useMemo(() => {
        if (!isChainSupported) {return null;}
        try {
            return getAddress(getPaymentReceiverAddress(chainId));
        } catch (_err) {
            return null;
        }
    }, [chainId, isChainSupported]);

    // Get token address for selected token
     
    const tokenAddress = useMemo(() => {
        if (!selectedToken || !isChainSupported) {return null;}
        try {
            return getAddress(getTokenAddress(selectedToken.symbol, chainId));
        } catch (_err) {
            return null;
        }
    }, [selectedToken, chainId, isChainSupported]);

    // Calculate amount in token decimals
    const amountInDecimals = useMemo(() => {
        if (!paymentData || !selectedToken) {return 0n;}
        // Use 18 decimals for USDT/USDC on BSC (they use 18, not 6)
        return parseUnits(paymentData.amount.toString(), selectedToken.decimals);
    }, [paymentData, selectedToken]);

    // ============ Direct Transfer Hook ============
    const {
        transfer,
        txHash,
        isTransferring,
        isConfirming: isPaymentConfirming,
        isConfirmed: isPaymentConfirmed
    } = useDirectTokenTransfer({
        tokenAddress,
        receiverAddress,
        amount: amountInDecimals,
        onError: (msg: string) => setError(msg)
    });

    // Get backend URL from environment
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
    const apiClient = createFrontendApiClient();

    // Fetch payment context data
    // eslint-disable-next-line complexity
    const fetchPaymentData = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            let data: PaymentLinkData | null = null;

            if (context.linkSlug !== null && context.linkSlug !== undefined) {
                // Fetch by dynamic link slug
                const response = await apiClient.get<PaymentLinkData>(`/api/public/payment-links/${context.linkSlug}`);
                if (response.success && response.data !== null && response.data !== undefined) {
                    data = response.data;
                } else {
                    // 404/410 errors are often thrown by the client, checking just in case
                    throw new Error('Failed to fetch payment link');
                }
            } else if (context.planId !== null && context.planId !== undefined) {
                // Fetch plan details
                interface PlanResponse {
                    id: string;
                    name: string;
                    description?: string;
                    price: number;
                }
                const response = await apiClient.get<PlanResponse>(`/api/public/plans/${context.planId}`);
                if (response.success && response.data !== null && response.data !== undefined) {
                    const plan = response.data;
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
                } else {
                    throw new Error('Plan not found');
                }
            } else if (context.groupId !== null && context.groupId !== undefined) {
                // Fetch group details
                interface GroupResponse {
                    id: string;
                    name: string;
                    description?: string;
                    price?: number;
                }
                const response = await apiClient.get<GroupResponse>(`/api/permissions/plans/${context.groupId}`);
                if (response.success && response.data !== null && response.data !== undefined) {
                    const group = response.data;
                    data = {
                        id: group.id,
                        context_type: 'group',
                        context_id: group.id,
                        slug: `group-${group.id}`,
                        name: group.name,
                        description: group.description,
                        amount: group.price ?? 0,
                        currency: 'USDT',
                        is_active: true,
                        is_usable: true,
                        current_uses: 0,
                        link_hash: ZERO_LINK_HASH,
                    };
                } else {
                    throw new Error('Group not found');
                }
            }

            setPaymentData(data);

            // Set default token
            if (supportedTokens.length > 0 && !selectedToken) {
                const defaultToken = supportedTokens.find(t => t.symbol === data?.currency) ?? supportedTokens[0];
                setSelectedToken(defaultToken);
            }
        } catch (err: unknown) {
            // Handle specific status codes if the error object has them
            const errObj = err as Record<string, unknown>;
            const status = errObj?.status;
            const message = errObj?.message;
            if (status === 404) {
                if (context.linkSlug !== null && context.linkSlug !== undefined) {setError('Payment link not found');}
                else {setError('Resource not found');}
            } else if (status === 410) {
                setError('Payment link has expired or reached max uses');
            } else {
                setError(typeof message === 'string' ? message : 'Failed to load payment details');
            }
        } finally {
            setLoading(false);
        }
        // Only depend on context and backendUrl to prevent infinite loops
    }, [context.linkSlug, context.planId, context.groupId, backendUrl]);

    useEffect(() => {
        if (context.linkSlug || context.planId || context.groupId) {
            fetchPaymentData();
        } else {
            setLoading(false);
            setError('No payment context provided');
        }
    }, [context, fetchPaymentData]);

    // ============ Backend Submission Logic ============
    // State for backend submission
    const [submissionStep, setSubmissionStep] = useState<'idle' | 'submitting' | 'submitted' | 'confirmed'>('idle');
    const _hasSubmittedRef = useState(false); // Using ref-like state or ref to prevent duplicate submissions
    // actually, let's use a real ref for submission tracking to avoid re-renders triggering it
    const submittedRef = useMemo(() => ({ current: false }), []);

    // Poll backend for transaction status
     
    const pollBackendStatus = useCallback((hash: string) => {
        const intervalId = setInterval(async () => {
            try {
                // Use apiClient
                // Backend returns wrapped response: { success: boolean, data: { status: string, ... } }
                const response = await apiClient.get<{ status: string; error_message?: string }>(`/api/payments/status/${hash}`);

                if (response.success && response.data) {
                    // Access nested data from backend wrapper
                    const statusData = response.data;

                    if (statusData) {
                        if (statusData.status === 'confirmed') {
                            clearInterval(intervalId);
                            setSubmissionStep('confirmed');
                            devLog('✅ Payment confirmed by backend!');
                            onPaymentSuccess?.(hash);
                        } else if (statusData.status === 'failed') {
                            clearInterval(intervalId);
                            setError(statusData.error_message ?? 'Payment failed verification');
                        }
                    }
                }
            } catch (err: unknown) {
                // Check for 404 which means not yet processed
                const errObj = err as Record<string, unknown>;
                const status = errObj?.status;
                if (status === 404) {
                    devLog('⏳ [Debug] Transaction not yet processed by backend (404)');
                    return;
                }
            }
        }, 3000);

        return () => clearInterval(intervalId);
    }, [onPaymentSuccess]);

    // Submit to backend when payment is confirmed on-chain
    useEffect(() => {
        if (isPaymentConfirmed && txHash && !submittedRef.current) {
            const submitTransaction = async () => {
                submittedRef.current = true;
                setSubmissionStep('submitting');

                try {
                    const requestBody = {
                        transaction_hash: txHash,
                        // context_id is the plan/group ID to pass to backend
                        plan_id: paymentData?.context_id,
                        expected_amount: paymentData?.amount,
                        currency: paymentData?.currency,
                        network: supportedChains.find(c => c.id === chainId)?.name ?? 'unknown'
                    };

                    // Use apiClient.post
                    const response = await apiClient.post('/api/payments/submit', requestBody);

                    if (response.success) {
                        setSubmissionStep('submitted');
                        // Start polling
                        pollBackendStatus(txHash);
                    } else {
                        setError(response.error?.message ?? 'Failed to submit payment to backend');
                    }
                } catch (err: unknown) {
                    const errObj = err as Record<string, unknown>;
                    const message = errObj?.message;
                    setError(`Backend submission failed: ${typeof message === 'string' ? message : 'Unknown error'}`);
                }
            };

            submitTransaction();
        }
    }, [isPaymentConfirmed, txHash, backendUrl, chainId, paymentData, pollBackendStatus, submittedRef]);

    // ============ Add Token to Wallet Hook ============
    const { addToken, isAdding: isAddingToken, isTokenAdded } = useAddTokenToWallet();

    // ============ Token Verification (for local Anvil) ============
    // Check if token contract is accessible - important for Tailscale IP access
    const [_verificationKey, _setVerificationKey] = useState(0);
    const { data: tokenSymbolData, isLoading: isVerifyingToken, isError: tokenVerifyError, refetch: recheckToken } = useReadContract({
        address: tokenAddress as `0x${string}`,
        abi: [{ name: 'symbol', type: 'function', inputs: [], outputs: [{ type: 'string' }] }] as const,
        functionName: 'symbol',
        chainId,
        query: {
            enabled: Boolean(tokenAddress) && chainId === 31337, // Only verify on local Anvil
            retry: 1,
            staleTime: 0,
        },
    });

    // Update verification status based on contract read result
    useEffect(() => {
        if (chainId !== 31337) {
            // Non-local chains are assumed to be verified
            setTokenVerificationStatus('verified');
        } else if (isVerifyingToken) {
            setTokenVerificationStatus('checking');
        } else if (tokenVerifyError) {
            setTokenVerificationStatus('failed');
        } else if (tokenSymbolData) {
            setTokenVerificationStatus('verified');
        } else {
            setTokenVerificationStatus('pending');
        }
    }, [chainId, isVerifyingToken, tokenVerifyError, tokenSymbolData]);

    // Handle recheck token button
    const handleRecheckToken = useCallback(() => {
        setTokenVerificationStatus('checking');
        _setVerificationKey(prev => prev + 1);
        recheckToken();
    }, [recheckToken]);

    // Check token balance (Wallet)
    const { data: balanceData } = useBalance({
        address,
        token: tokenAddress as `0x${string}`,
        chainId,
        query: {
            enabled: Boolean(address) && Boolean(tokenAddress),
        },
    });

    // Check token balance (Public/RPC Direct) - Bypass wallet RPC issues
    const { balance: publicBalance } = usePublicBalance(
        tokenAddress ?? undefined,
        address
    );

    // Start payment flow - Register token with MetaMask, then transfer
    const handlePayment = async () => {
        if (!paymentData || !selectedToken || !address || !receiverAddress || !tokenAddress) {
            setError('Missing required data for payment');
            return;
        }

        // Check balance before proceeding
        if (balanceData && balanceData.value < amountInDecimals) {
            setError(`Insufficient ${selectedToken.symbol} balance. You have ${balanceData.formatted} ${selectedToken.symbol}, but verify ${paymentData.amount} ${selectedToken.symbol} is required.`);
            return;
        }

        // Double-check chain support before payment
        if (!isChainSupported) {
            setError('Please switch to a supported network (BSC Mainnet, BSC Testnet, or Anvil Local)');
            return;
        }

        setError(null);

        // Step 1: Add token to wallet if not already added
        // This helps MetaMask recognize and decode the ERC20 transfer
        if (!isTokenAdded(selectedToken.symbol)) {
            devLog('📝 Registering token with MetaMask...');
            await addToken(selectedToken.symbol);
            // Continue even if user rejects - the transfer will still work
        }

        devLog('💸 Starting direct token transfer...');

        // Step 2: Execute direct transfer
        transfer();
    };

    // Determine current step for UI
    const currentStep = useMemo(() => {
        if (submissionStep === 'confirmed') {return 'complete';} // Only complete when backend confirms
        if (isTransferring || isPaymentConfirming || submissionStep === 'submitting' || submissionStep === 'submitted') {return 'paying';}
        if (isAddingToken) {return 'adding-token';}
        return 'idle';
    }, [isTransferring, isPaymentConfirming, isPaymentConfirmed, isAddingToken, submissionStep]);

    const isBusy = isAddingToken || isTransferring || isPaymentConfirming || submissionStep === 'submitting' || submissionStep === 'submitted';

    // Loading state
    if (loading) {
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="flex flex-col items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600" />
                    <p className="mt-4 text-gray-600 dark:text-gray-400">Loading payment details...</p>
                </div>
            </div>
        );
    }

    // Error state (only for initial load failures)
    if (!paymentData) {
        return (
            <div className={cn('bg-white dark:bg-gray-800 rounded-2xl p-8 shadow-xl', className)}>
                <div className="text-center py-12">
                    <div className="text-6xl mb-4">❌</div>
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                        Payment Unavailable
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                        {error ?? 'This payment link is no longer available'}
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

    // Chain verification state - show when connected but on wrong network
    if (isConnected && !isChainSupported) {
        return (
            <ChainVerificationCard className={className} />
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

            {/* Token Verification Status (for local Anvil) */}
            {chainId === 31337 && selectedToken && (
                <div className="mb-6 space-y-3">
                    <div className={cn(
                        'p-4 rounded-lg border flex items-center justify-between',
                        tokenVerificationStatus === 'verified'
                            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
                            : tokenVerificationStatus === 'failed'
                                ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                                : 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800'
                    )}>
                        <div className="flex items-center gap-2">
                            {tokenVerificationStatus === 'checking' && (
                                <Loader2 className="w-4 h-4 text-yellow-600 animate-spin" />
                            )}
                            {tokenVerificationStatus === 'verified' && (
                                <CheckCircle className="w-4 h-4 text-green-600" />
                            )}
                            {tokenVerificationStatus === 'failed' && (
                                <AlertTriangle className="w-4 h-4 text-red-600" />
                            )}
                            <span className={cn(
                                'text-sm font-medium',
                                tokenVerificationStatus === 'verified' ? 'text-green-700 dark:text-green-300'
                                    : tokenVerificationStatus === 'failed' ? 'text-red-700 dark:text-red-300'
                                        : 'text-yellow-700 dark:text-yellow-300'
                            )}>
                                {tokenVerificationStatus === 'checking' && `Checking ${selectedToken.symbol} token...`}
                                {tokenVerificationStatus === 'verified' && `${selectedToken.symbol} token verified ✓`}
                                {tokenVerificationStatus === 'failed' && `Cannot reach ${selectedToken.symbol} token contract`}
                                {tokenVerificationStatus === 'pending' && `Verifying ${selectedToken.symbol} token...`}
                            </span>
                        </div>
                        <button
                            onClick={handleRecheckToken}
                            disabled={tokenVerificationStatus === 'checking'}
                            className={cn(
                                'flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-all',
                                'bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600',
                                'hover:bg-gray-100 dark:hover:bg-gray-700',
                                tokenVerificationStatus === 'checking' && 'opacity-50 cursor-not-allowed'
                            )}
                        >
                            <RefreshCw className={cn('w-3 h-3', tokenVerificationStatus === 'checking' && 'animate-spin')} />
                            Recheck Token
                        </button>
                    </div>

                    {/* RPC Warning Mismatch */}
                    {publicBalance && balanceData && parseFloat(publicBalance) > parseFloat(balanceData.formatted) && (
                        <div className="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
                            <div className="flex items-start gap-3">
                                <AlertTriangle className="w-5 h-5 text-red-600 mt-0.5" />
                                <div>
                                    <h5 className="font-bold text-red-800 dark:text-red-200 text-sm">RPC Connection Issue Detected</h5>
                                    <p className="text-sm text-red-700 dark:text-red-300 mt-1">
                                        Your actual balance is <strong>{publicBalance} {selectedToken.symbol}</strong>, but MetaMask shows {balanceData.formatted}.
                                    </p>
                                    <p className="text-sm text-red-700 dark:text-red-300 mt-2">
                                        <strong>To fix this:</strong>
                                        <ol className="list-decimal ml-4 mt-1 space-y-1">
                                            <li>Open MetaMask Settings → Networks → Anvil Local</li>
                                            <li>Change RPC URL to: <code className="bg-red-100 dark:bg-red-900/40 px-1 py-0.5 rounded">http://{typeof window !== 'undefined' ? window.location.hostname : '127.0.0.1'}:8545</code></li>
                                            <li>Save and Refresh this page</li>
                                            <li>Remove {selectedToken.symbol} from MetaMask and add it again</li>
                                        </ol>
                                    </p>
                                </div>
                            </div>
                        </div>
                    )}
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

            {/* Error Display */}
            {error && (
                <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
                    <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
                </div>
            )}

            {/* Add Token to Wallet Step - Show before payment if token not added */}
            {isConnected && selectedToken && !isTokenAdded(selectedToken.symbol) && submissionStep !== 'confirmed' && (
                <div className="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
                    <div className="flex items-start gap-3">
                        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-800 flex items-center justify-center">
                            <span className="text-blue-600 dark:text-blue-300 font-bold text-sm">1</span>
                        </div>
                        <div className="flex-1">
                            <h4 className="font-semibold text-blue-800 dark:text-blue-200 mb-1">
                                Add {selectedToken.symbol} to Wallet
                            </h4>
                            <p className="text-sm text-blue-700 dark:text-blue-300 mb-2">
                                First, add {selectedToken.symbol} token to your wallet to view your balance and enable payments.
                            </p>
                            {/* Show on-chain balance */}
                            {(publicBalance ?? balanceData) && (
                                <div className="mb-3">
                                    <p className="text-sm text-green-700 dark:text-green-300 font-medium">
                                        ✅ On-chain balance: {publicBalance ?? balanceData?.formatted} {selectedToken.symbol}
                                    </p>

                                    {/* Show funding help if balance is 0 */}
                                    {((publicBalance && parseFloat(publicBalance) === 0) ?? (!publicBalance && balanceData && parseFloat(balanceData.formatted) === 0)) && (
                                        <div className="mt-2 text-xs bg-yellow-50 dark:bg-yellow-900/10 p-2 rounded border border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200">
                                            <strong>Need test tokens?</strong> Run this in your terminal:
                                            <code className="block mt-1 bg-white dark:bg-black/20 p-1 rounded select-all">
                                                bun fund:wallet {address}
                                            </code>
                                        </div>
                                    )}
                                </div>
                            )}

                            {/* Warning tip if mismatch */}
                            {publicBalance && balanceData && parseFloat(publicBalance) > 0 && parseFloat(balanceData.formatted) === 0 && (
                                <p className="text-xs text-blue-600 dark:text-blue-400 mb-3">
                                    💡 MetaMask shows "0" but you have funds. This is a connection issue. See the red box below for the fix.
                                </p>
                            )}
                            <button
                                onClick={() => addToken(selectedToken.symbol)}
                                disabled={isAddingToken}
                                className={cn(
                                    'px-4 py-2 rounded-lg font-medium text-sm transition-all',
                                    isAddingToken
                                        ? 'bg-blue-300 cursor-not-allowed'
                                        : 'bg-blue-600 hover:bg-blue-700 text-white'
                                )}
                            >
                                {isAddingToken ? (
                                    <span className="flex items-center gap-2">
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                        Adding Token...
                                    </span>
                                ) : (
                                    `Add ${selectedToken.symbol} to Wallet`
                                )}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Local Anvil Diagnostics - ALWAYS SHOW if on local chain */}
            {chainId === 31337 && selectedToken && (
                <div className="mb-6 space-y-3">
                    {/* On-chain balance display */}
                    {(publicBalance ?? balanceData) && (
                        <div className="p-3 bg-gray-50 dark:bg-white/5 rounded-lg border border-gray-200 dark:border-white/10">
                            <p className="text-sm font-medium flex items-center gap-2">
                                <span>🔍 Diagnostics:</span>
                                <span className={cn(
                                    publicBalance && parseFloat(publicBalance) > 0 ? "text-green-600" : "text-gray-600"
                                )}>
                                    Real On-Chain Balance: {publicBalance ?? '...'} {selectedToken.symbol}
                                </span>
                            </p>
                            <p className="text-xs text-gray-500 mt-1">
                                MetaMask sees: {balanceData?.formatted ?? '0'} {selectedToken.symbol}
                            </p>
                        </div>
                    )}

                    {/* Warning tip if mismatch */}
                    {publicBalance && balanceData && parseFloat(publicBalance) > 0 && parseFloat(balanceData.formatted) === 0 && (
                        <div className="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
                            <div className="flex items-start gap-3">
                                <AlertTriangle className="w-5 h-5 text-red-600 mt-0.5" />
                                <div>
                                    <h5 className="font-bold text-red-800 dark:text-red-200 text-sm">RPC Connection Issue Detected</h5>
                                    <p className="text-sm text-red-700 dark:text-red-300 mt-1">
                                        Your actual balance is <strong>{publicBalance} {selectedToken.symbol}</strong>, but MetaMask shows {balanceData.formatted}.
                                    </p>
                                    <p className="text-sm text-red-700 dark:text-red-300 mt-2">
                                        <strong>To fix this:</strong>
                                        <ol className="list-decimal ml-4 mt-1 space-y-1">
                                            <li>Open MetaMask Settings → Networks → Anvil Local</li>
                                            <li>Change RPC URL to: <code className="bg-red-100 dark:bg-red-900/40 px-1 py-0.5 rounded">http://{typeof window !== 'undefined' ? window.location.hostname : '127.0.0.1'}:8545</code></li>
                                            <li>Save and Refresh this page</li>
                                            <li>Remove {selectedToken.symbol} from MetaMask and add it again</li>
                                        </ol>
                                    </p>
                                </div>
                            </div>
                        </div>
                    )}
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
            ) : submissionStep === 'confirmed' ? (
                <button
                    disabled
                    className="w-full py-4 rounded-xl font-bold text-lg bg-green-500 text-white flex items-center justify-center gap-2"
                >
                    <CheckCircle className="w-5 h-5" />
                    Payment Successful!
                </button>
            ) : !selectedToken || !isTokenAdded(selectedToken.symbol) ? (
                /* Disabled Pay Button - Token not added yet */
                <button
                    disabled
                    className="w-full py-4 rounded-xl font-bold text-lg bg-gray-300 dark:bg-gray-700 text-gray-500 dark:text-gray-400 cursor-not-allowed"
                >
                    Add Token to Wallet First
                </button>
            ) : (
                <button
                    onClick={handlePayment}
                    disabled={isBusy || !selectedToken}
                    className={cn(
                        'w-full py-4 rounded-xl font-bold text-lg transition-all',
                        isBusy
                            ? 'bg-gray-400 cursor-not-allowed'
                            : 'bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white shadow-lg hover:shadow-xl'
                    )}
                >
                    {isBusy ? (
                        <span className="flex items-center justify-center gap-2">
                            <Loader2 className="w-5 h-5 animate-spin" />
                            {currentStep === 'adding-token' ? 'Adding Token to Wallet...' :
                                isPaymentConfirming ? 'Confirming...' :
                                    submissionStep === 'submitting' || submissionStep === 'submitted' ? 'Verifying...' : 'Processing...'}
                        </span>
                    ) : (
                        `Pay ${paymentData.amount} ${selectedToken?.symbol || paymentData.currency}`
                    )}
                </button>
            )}

            {/* Transaction Success */}
            {submissionStep === 'confirmed' && txHash && (
                <div className="mt-6 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
                    <div className="flex items-center">
                        <CheckCircle className="w-6 h-6 text-green-500 mr-3" />
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
