
'use client';

/**
 * UnifiedPaymentFlow Component
 * 
 * Complete payment flow with:
 * - Current access display
 * - Plan selection with upgrade-only logic
 * - Chain verification
 * - Token management
 * - Payment execution
 */

import { submitTransactionAction } from '@/app/actions/payments';
import { getPublicPlansAction } from '@/app/actions/plans';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import {
    AlertCircle,
    ArrowLeft,
    Check,
    CheckCircle2,
    Loader2,
    Lock,
    Shield,
    Sparkles,
    Wallet,
    Zap
} from 'lucide-react';
import Link from 'next/link';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { getAddress, parseUnits } from 'viem';
import { useAccount, useBalance, useChainId } from 'wagmi';

import { usePlanAccess } from '@/hooks/use-plan-access';
import { useAuth } from '@/lib/auth';
import { getPaymentReceiverAddress, getTokenAddress } from '@/lib/contracts/addresses';
import { cn } from '@/lib/utils';
import { supportedChains } from '@/shared/components/navigation/chain-selector';
import { logger } from '@/shared/utils/logger';

import { ChainVerificationCard } from './chain-verification-card';
import { CurrentAccessCard } from './current-access-card';
import { useAddTokenToWallet } from './hooks/use-add-token-to-wallet';
import { useDirectTokenTransfer } from './hooks/use-direct-token-transfer';

// Shared Components
import type { PricingCardData } from '@/shared/components/plans/pricing-card';
import { PricingCard } from '@/shared/components/plans/pricing-card';

// Types
type PaymentType = 'plan' | 'access-plan' | 'permission';
type PaymentStep = 'select' | 'confirm' | 'pay' | 'success';

interface RawPlan {
    id: string;
    name: string;
    current_price: string | number;
    base_price?: string | number;
    is_active: boolean;
    display_order?: number;
    is_highlighted?: boolean;
    is_promoted?: boolean;
    features?: string[] | string;
    tier_level?: number;
    plan_type?: string;
    description?: string;
}

interface UnifiedPaymentFlowProps {
    /** Payment type determines which endpoint to use */
    paymentType: PaymentType;
    /** Pre-selected item ID (plan/group/permission UUID) */
    preselectedId?: string;
    /** Custom title */
    title?: string;
    /** Custom description */
    description?: string;
    className?: string;
    initialPlans?: RawPlan[];
}

/**
 * Supported token type for payment
 */
interface PaymentToken {
    symbol: string;
    name: string;
    decimals: number;
}

const PAYMENT_TOKENS: PaymentToken[] = [
    { symbol: 'USDT', name: 'Tether USD', decimals: 18 },
    { symbol: 'USDC', name: 'USD Coin', decimals: 18 },
    { symbol: 'DAI', name: 'Dai Stablecoin', decimals: 18 },
];

export function UnifiedPaymentFlow({
    paymentType,
    preselectedId,
    title,
    description,
    className,
    initialPlans = [],
}: UnifiedPaymentFlowProps) {
    // Auth and wallet state
    const { user, isLoading: isAuthLoading, openSignInModal } = useAuth();
    const { address, isConnected } = useAccount();
    const chainId = useChainId();
    const isAuthenticated = Boolean(user);

    // Plan access hook
    const { planAccess, loading: planAccessLoading, refetch: refetchPlanAccess } = usePlanAccess();

    // Payment flow state
    const [step, setStep] = useState<PaymentStep>('select');
    const [plans, setPlans] = useState<PricingCardData[]>([]);
    const [selectedPlan, setSelectedPlan] = useState<PricingCardData | null>(null);
    const [selectedToken, setSelectedToken] = useState<PaymentToken>(PAYMENT_TOKENS[0]);
    // Allow loading to be false if we have initial plans
    const [loading, setLoading] = useState(initialPlans.length === 0);
    const [error, setError] = useState<string | null>(null);
    const [txHash, setTxHash] = useState<string | null>(null);

    // Internal fetch state to trigger re-fetch on retry (reserved for future use)

    // State to toggle between single view and grid view if user wants to change plan
    const [showAllPlans, setShowAllPlans] = useState(false);

    // API client
    const apiClient = useMemo(() => createFrontendApiClient(), []);

    // Check chain support
    const isChainSupported = useMemo(
        () => supportedChains.some(chain => chain.id === chainId),
        [chainId]
    );

    // Token address for selected token

    const tokenAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try {
            return getAddress(getTokenAddress(selectedToken.symbol, chainId));
        } catch (_err) {
            return null;
        }
    }, [selectedToken, chainId, isChainSupported]);

    // Token balance
    const { data: balanceData } = useBalance({
        address,
        token: tokenAddress as `0x${string}`,
        chainId,
        query: { enabled: Boolean(address) && Boolean(tokenAddress) },
    });

    // Receiver address

    const receiverAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try {
            return getAddress(getPaymentReceiverAddress(chainId));
        } catch (_err) {
            return null;
        }
    }, [chainId, isChainSupported]);

    // Amount in token decimals
    const amountInDecimals = useMemo(() => {
        if (!selectedPlan) { return 0n; }
        // Parse price string (remove $, USD, etc)
        const priceVal = selectedPlan.price.replace(/[^0-9.]/g, '');
        return parseUnits(priceVal, selectedToken.decimals);
    }, [selectedPlan, selectedToken]);

    // Add token to wallet hook
    const { addToken, isAdding: isAddingToken, isTokenAdded } = useAddTokenToWallet();

    // Direct token transfer hook

    const {
        transfer,
        txHash: transferTxHash,
        isTransferring,
        isConfirming,
        isConfirmed
    } = useDirectTokenTransfer({
        tokenAddress,
        receiverAddress,
        amount: amountInDecimals,

        onError: (msg: any) => {

            const errorMsg = msg;
            setError(typeof errorMsg === 'string' ? errorMsg : 'An error occurred');
            // Reset to confirm step so user can see the error and try again
            setStep('confirm');
        }
    });

    // Current plan tier level - use tier_level from planAccess if available
    const currentPlanTier = useMemo(() => {
        const plan = planAccess?.plan_name;
        if (!plan) { return 0; }
        // planAccess.tier_level should come from the backend
        // If not available yet, default to 0 (will be added to backend response)

        const tierLevel = (planAccess as any)?.tier_level;
        return typeof tierLevel === 'number' ? tierLevel : 0;
    }, [planAccess]);

    // Is current plan expired?
    const isExpired = planAccess?.status === 'expired';

    // Helper to calculate action type for a plan
    const getActionType = useCallback((plan: PricingCardData) => {
        const planTier = typeof plan.tier_level === 'number' ? plan.tier_level : 0;

        if (planTier === currentPlanTier) { return 'extend'; }
        if (planTier > currentPlanTier) { return 'upgrade'; }
        if (planTier < currentPlanTier) {
            return 'downgrade';
        }
        return 'select';
    }, [currentPlanTier]);

    // Transform plans helper
    const transformPlans = useCallback((rawPlans: RawPlan[]): PricingCardData[] => {
        return rawPlans
            .filter((plan) => plan.is_active)
            .sort((a, b) => (a.display_order ?? 0) - (b.display_order ?? 0))
            .map((plan) => {
                const price = typeof plan.current_price === 'string'
                    ? parseFloat(plan.current_price)
                    : plan.current_price;

                const basePrice = plan.base_price
                    ? (typeof plan.base_price === 'string' ? parseFloat(plan.base_price) : plan.base_price)
                    : null;

                const isFree = price === 0;

                const parsedFeatures = (Array.isArray(plan.features)
                    ? plan.features
                    : typeof plan.features === 'string'
                        ? JSON.parse(plan.features)
                        : []) as Array<string | { text: string; included: boolean }>;

                return {
                    id: plan.id,
                    title: plan.name.replace(/\s+Plan$/i, ''),
                    price: isFree ? 'Free' : `${price}`,
                    originalPrice: basePrice ? `$${basePrice}` : undefined,
                    features: parsedFeatures.map((f) => typeof f === 'string' ? { text: f, included: true } : f),
                    highlight: plan.is_highlighted ?? plan.is_promoted,
                    buttonText: isFree ? 'Start Free' : 'Select Plan',
                    tier_level: plan.tier_level ?? 0,
                    plan_type: plan.plan_type,
                    description: plan.description
                };
            });
    }, []);

    // Function to fetch plans manually (for retry)
    const fetchPlans = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const result = await getPublicPlansAction();

            if (result.success && result.data && Array.isArray(result.data)) {
                const transformed = transformPlans(result.data);
                setPlans(transformed);
                // Auto-select preselected plan
                if (preselectedId) {
                    const preselected = transformed.find((p) => String(p.id) === String(preselectedId));
                    if (preselected) {
                        setSelectedPlan(preselected);
                    }
                }
            } else {
                const errorMsg = result.error?.message ?? 'Invalid API response format';
                throw new Error(errorMsg);
            }
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : 'Failed to load plans';
            setError(`${errorMsg}. Please refresh or try again.`);
            logger.error('[Payment] Failed to fetch plans:', err);
        } finally {
            setLoading(false);
        }
    }, [preselectedId, transformPlans]);

    // Initialize/Update plans from props or fetch if missing
    useEffect(() => {
        if (initialPlans.length > 0) {
            const transformed = transformPlans(initialPlans);
            setPlans(transformed);
            setLoading(false);

            // Auto-select preselected plan
            if (preselectedId) {
                const preselected = transformed.find((p) => String(p.id) === String(preselectedId));
                if (preselected) {
                    setSelectedPlan(preselected);
                }
            }
        } else {
            // Fetch if no initial plans
            void fetchPlans();
        }
    }, [initialPlans, preselectedId, transformPlans, fetchPlans]);

    // Safety timeout to prevent infinite loading
    useEffect(() => {
        const timeout = setTimeout(() => {
            if (loading) {
                setError('Loading timeout - please refresh the page');
                setLoading(false);
            }
        }, 10000); // 10 second timeout

        return () => clearTimeout(timeout);
    }, [loading]);

    // Handle plan selection
    const handlePlanSelect = (plan: PricingCardData) => {
        setSelectedPlan(plan);
        setError(null);
        // If clicking on a plan in grid, view details/confirm
        // If clicking on single view, go to confirm
        if (preselectedId && !showAllPlans && selectedPlan?.id === plan.id) {
            setStep('confirm');
        } else if (!preselectedId) {
            // If no preselection (standard /plans page), maybe selecting keeps it there or goes to confirm?
            // The old behavior was: select -> set state. Then click "Continue".
            // With PricingCard, the button inside clicks 'onSelect'. 
            // Let's go to confirm immediately for better UX
            setStep('confirm');
        } else {
            // We successfully selected a plan (maybe changed from grid)
            // Go to confirm
            setStep('confirm');
        }
    };

    // Proceed to confirmation
    const handleProceedToConfirm = () => {
        if (!selectedPlan) {
            setError('Please select a plan');
            return;
        }
        setStep('confirm');
    };

    // Handle payment
    const handlePayment = async () => {
        // Validate requirements with specific error messages
        if (!selectedPlan) {
            setError('Please select a plan before proceeding');
            return;
        }
        if (!address) {
            setError('Wallet not connected. Please connect your wallet.');
            return;
        }
        if (!receiverAddress) {
            setError(`Payment receiver not configured for chain ${chainId}. Please switch to a supported network.`);
            return;
        }
        if (!tokenAddress) {
            setError(`${selectedToken.symbol} token not available on chain ${chainId}. Please switch to a supported network.`);
            return;
        }

        if (balanceData && balanceData.value < amountInDecimals) {
            setError(`Insufficient ${selectedToken.symbol} balance. You have ${balanceData.formatted} ${selectedToken.symbol}, but ${selectedPlan.price.replace(/[^0-9.]/g, '')} is required.`);
            return;
        }

        setError(null);
        setStep('pay');

        // Add token to wallet if not already added
        if (!isTokenAdded(selectedToken.symbol)) {
            await addToken(selectedToken.symbol);
        }

        // Execute transfer
        transfer();
    };

    // Backend submission after payment confirmed

    useEffect(() => {
        if (isConfirmed && transferTxHash && step === 'pay') {
            // Submit to backend
            const submitPayment = async () => {
                try {
                    // Extract numeric price
                    const priceVal = parseFloat(selectedPlan?.price.replace(/[^0-9.]/g, '') ?? '0');

                    const result = await submitTransactionAction({
                        transaction_hash: transferTxHash,
                        plan_id: selectedPlan?.id as string,
                        expected_amount: priceVal,
                        currency: selectedToken.symbol,
                        // network: supportedChains.find(c => c.id === chainId)?.name || 'unknown' // Backend might not take this in submitTransactionAction type
                    });

                    if (result.success) {
                        setTxHash(transferTxHash);
                        setStep('success');
                        // Refresh plan access

                        refetchPlanAccess();
                    } else {
                        setError(result.error?.message ?? 'Payment submitted but verification pending');
                    }
                } catch (_err) {
                    setError('Payment confirmed but backend submission failed');
                }
            };

            submitPayment();
        }
    }, [isConfirmed, transferTxHash, step, selectedPlan, selectedToken, chainId, apiClient, refetchPlanAccess]);

    // Get page title based on payment type
    const getPageTitle = () => {
        if (title) { return title; }
        switch (paymentType) {
            case 'plan': return 'Choose Your Plan';
            case 'access-plan': return 'Join Group';
            case 'permission': return 'Unlock Access';
            default: return 'Make Payment';
        }
    };

    const getPageDescription = () => {
        if (description) { return description; }
        switch (paymentType) {
            case 'plan': return 'Select a plan to unlock premium features and analytics';
            case 'access-plan': return 'Join this group to access shared permissions';
            case 'permission': return 'Purchase access to this specific feature';
            default: return 'Complete your payment securely';
        }
    };

    // Loading state
    // Only wait for plans and access if we are authenticated. 
    // If not authenticated, we will show the sign-in screen immediately after auth check completes.
    if (isAuthLoading || (isAuthenticated && (loading || planAccessLoading))) {
        return (
            <div className={cn('py-12', className)}>
                <div className="flex flex-col items-center justify-center gap-4">
                    <Loader2 className="w-12 h-12 text-blue-500 animate-spin" />
                    <p className="text-gray-600 dark:text-gray-400">Loading payment options...</p>
                </div>
            </div>
        );
    }

    // Not authenticated
    if (!isAuthenticated) {
        return (
            <div className={cn('max-w-lg mx-auto', className)}>
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-8 shadow-xl border border-blue-200/50 dark:border-blue-700/50 text-center">
                    <div className="flex h-20 w-20 items-center justify-center rounded-full bg-amber-100 dark:bg-amber-900/30 mx-auto mb-6">
                        <Wallet className="h-10 w-10 text-amber-600 dark:text-amber-400" />
                    </div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                        Sign In to Continue
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-6">
                        {isConnected
                            ? 'Please sign the message to verify your wallet.'
                            : 'Please connect your wallet to proceed with payment.'}
                    </p>
                    <div className="bg-gray-50 dark:bg-gray-800 rounded-xl p-4 mb-6 text-left">
                        <div className="flex items-start gap-3 mb-3">
                            <Shield className="h-5 w-5 text-green-500 mt-0.5" />
                            <div>
                                <p className="font-medium text-gray-900 dark:text-white">Secure Authentication</p>
                                <p className="text-sm text-gray-500 dark:text-gray-400">Sign-In with Ethereum (SIWE)</p>
                            </div>
                        </div>
                        <div className="flex items-start gap-3">
                            <Lock className="h-5 w-5 text-blue-500 mt-0.5" />
                            <div>
                                <p className="font-medium text-gray-900 dark:text-white">One-Time Sign In</p>
                                <p className="text-sm text-gray-500 dark:text-gray-400">Stay signed in until you disconnect</p>
                            </div>
                        </div>
                    </div>
                    <button
                        onClick={openSignInModal}
                        className="w-full py-3.5 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-blue-900/20 active:scale-[0.98]"
                    >
                        {isConnected ? 'Sign In' : 'Connect Wallet'}
                    </button>
                </div>
            </div>
        );
    }

    // Chain verification needed
    if (isConnected && !isChainSupported) {
        return <ChainVerificationCard className={className} />;
    }

    // Error state
    if (error && plans.length === 0) {
        return (
            <div className={cn('max-w-lg mx-auto text-center', className)}>
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-8 shadow-xl border border-red-200/50 dark:border-red-700/50">
                    <div className="flex h-16 w-16 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/30 mx-auto mb-4">
                        <AlertCircle className="h-8 w-8 text-red-600 dark:text-red-400" />
                    </div>
                    <h2 className="text-xl font-bold text-red-600 dark:text-red-400 mb-2">
                        Failed to Load Plans
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-4">{error}</p>
                    <button
                        onClick={fetchPlans}
                        className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        Try Again
                    </button>
                </div>
            </div>
        );
    }

    // Success step
    if (step === 'success') {
        return (
            <div className={cn('max-w-lg mx-auto text-center', className)}>
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-8 shadow-xl border border-green-200/50 dark:border-green-700/50">
                    <div className="flex h-20 w-20 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/30 mx-auto mb-6">
                        <CheckCircle2 className="h-10 w-10 text-green-600 dark:text-green-400" />
                    </div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                        🎉 Payment Successful!
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 mb-6">
                        Your {selectedPlan?.title} plan is now active.
                    </p>

                    {txHash && (
                        <div className="bg-gray-50 dark:bg-gray-800 rounded-xl p-4 mb-6">
                            <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">Transaction Hash</p>
                            <code className="text-xs font-mono text-gray-700 dark:text-gray-300 break-all">
                                {txHash}
                            </code>
                        </div>
                    )}

                    <div className="flex flex-col sm:flex-row gap-3">
                        <Link
                            href="/account"
                            className="flex-1 px-6 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl font-bold hover:shadow-lg transition-all flex items-center justify-center gap-2"
                        >
                            <Wallet className="w-4 h-4" />
                            View Account
                        </Link>
                        <Link
                            href="/analytics"
                            className="flex-1 px-6 py-3 border-2 border-blue-300 dark:border-blue-700 text-blue-600 dark:text-blue-400 rounded-xl font-bold hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all"
                        >
                            Go to Analytics
                        </Link>
                    </div>
                </div>
            </div>
        );
    }

    // Confirmation step
    if (step === 'confirm' && selectedPlan) {
        return (
            <div className={cn('max-w-2xl mx-auto', className)}>
                {/* Back button */}
                <button
                    onClick={() => setStep('select')}
                    className="flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 mb-6 transition-colors"
                >
                    <ArrowLeft className="w-4 h-4" />
                    Back to plans
                </button>

                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-8 shadow-xl border border-blue-200/50 dark:border-blue-700/50">
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
                        Confirm Your Order
                    </h2>

                    {/* Order summary */}
                    <div className="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-xl p-6 mb-6">
                        <div className="flex items-center justify-between mb-4">
                            <div>
                                <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                                    {selectedPlan.title}
                                </h3>
                                <p className="text-sm text-gray-600 dark:text-gray-400">
                                    30 days access
                                </p>
                            </div>
                            <div className="text-right">
                                <p className="text-2xl font-black text-blue-600 dark:text-blue-400">
                                    ${selectedPlan.price.replace(/[^0-9.]/g, '')}
                                </p>
                                <p className="text-xs text-gray-500 dark:text-gray-400">
                                    {selectedToken.symbol}
                                </p>
                                {balanceData && (
                                    <p className="text-xs text-gray-500 dark:text-gray-400">
                                        Balance: {balanceData.formatted} {selectedToken.symbol}
                                    </p>
                                )}
                            </div>
                        </div>

                        {/* Features preview */}
                        <div className="border-t border-blue-200 dark:border-blue-800 pt-4">
                            <p className="text-xs font-bold text-gray-500 dark:text-gray-400 mb-2 uppercase">
                                Included Features
                            </p>
                            <div className="grid grid-cols-2 gap-2">
                                {selectedPlan.features.slice(0, 4).map((feature) => (
                                    <div key={feature.text} className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                                        <Check className="w-3 h-3 text-green-500" />
                                        <span className="truncate">{feature.text}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>

                    {/* Token selection */}
                    <div className="mb-6">
                        <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Pay with
                        </p>
                        <div className="flex gap-2">
                            {PAYMENT_TOKENS.map((token) => (
                                <button
                                    key={token.symbol}
                                    onClick={() => setSelectedToken(token)}
                                    className={cn(
                                        'flex-1 px-4 py-3 rounded-xl border-2 transition-all font-medium',
                                        selectedToken.symbol === token.symbol
                                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
                                            : 'border-gray-200 dark:border-gray-700 hover:border-blue-300'
                                    )}
                                >
                                    {token.symbol}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Add token prompt */}
                    {!isTokenAdded(selectedToken.symbol) && (
                        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4 mb-6">
                            <div className="flex items-start gap-3">
                                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-800 flex items-center justify-center">
                                    <Zap className="w-4 h-4 text-blue-600 dark:text-blue-300" />
                                </div>
                                <div className="flex-1">
                                    <p className="font-medium text-blue-800 dark:text-blue-200 mb-1">
                                        Add {selectedToken.symbol} to Wallet
                                    </p>
                                    <p className="text-sm text-blue-600 dark:text-blue-300">
                                        We'll prompt you to add the token when you proceed.
                                    </p>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Error */}
                    {error && (
                        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 mb-6">
                            <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
                        </div>
                    )}

                    {/* Pay button */}
                    <button
                        onClick={handlePayment}
                        disabled={isAddingToken || isTransferring || isConfirming || (Boolean(balanceData) && balanceData!.value < amountInDecimals)}
                        className={cn(
                            'w-full py-4 rounded-xl font-bold text-lg transition-all',
                            isAddingToken || isTransferring || isConfirming || (Boolean(balanceData) && balanceData!.value < amountInDecimals)
                                ? 'bg-gray-300 dark:bg-gray-700 cursor-not-allowed'
                                : 'bg-gradient-to-r from-blue-600 to-indigo-600 text-white hover:shadow-xl shadow-lg shadow-blue-500/30'
                        )}
                    >
                        {isAddingToken ? (
                            <span className="flex items-center justify-center gap-2">
                                <Loader2 className="w-5 h-5 animate-spin" />
                                Adding Token...
                            </span>
                        ) : isConfirming ? (
                            <span className="flex items-center justify-center gap-2">
                                <Loader2 className="w-5 h-5 animate-spin" />
                                Confirming...
                            </span>
                        ) : isTransferring ? (
                            <span className="flex items-center justify-center gap-2">
                                <Loader2 className="w-5 h-5 animate-spin" />
                                Confirm in Wallet...
                            </span>
                        ) : (Boolean(balanceData) && balanceData!.value < amountInDecimals) ? (
                            <span className="flex items-center justify-center gap-2">
                                <AlertCircle className="w-5 h-5" />
                                Insufficient {selectedToken.symbol} Balance
                            </span>
                        ) : (
                            <span className="flex items-center justify-center gap-2">
                                <Wallet className="w-5 h-5" />
                                Pay ${selectedPlan.price.replace(/[^0-9.]/g, '')} {selectedToken.symbol}
                            </span>
                        )}
                    </button>

                    <p className="text-center text-xs text-gray-500 dark:text-gray-400 mt-4">
                        By proceeding, you agree to our Terms of Service
                    </p>
                </div>
            </div>
        );
    }

    // Single Plan Selection Mode
    if (preselectedId && selectedPlan && !showAllPlans && step === 'select') {
        return (
            <div className={cn('space-y-8', className)}>
                <div className="text-center">
                    <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-2">
                        {selectedPlan.title} Plan
                    </h2>
                    <p className="text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                        Review plan details before proceeding
                    </p>
                </div>

                <div className="max-w-md mx-auto">
                    <PricingCard
                        card={{ ...selectedPlan, buttonText: "Proceed to Payment" }}
                        onSelect={handlePlanSelect}
                        isSelected={true}
                        actionType="select"
                        isDisabled={false}
                    />

                    <div className="text-center mt-8">
                        <button
                            onClick={() => setShowAllPlans(true)}
                            className="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-500 underline transition-colors"
                        >
                            View other plans
                        </button>
                    </div>
                </div>
            </div>
        )
    }

    // Main selection step (Grid View)
    return (
        <div className={cn('space-y-8', className)}>
            {/* Current Access */}
            <CurrentAccessCard paymentType={paymentType} />

            {/* Header */}
            <div className="text-center">
                <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-2">
                    {getPageTitle()}
                </h2>
                <p className="text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                    {getPageDescription()}
                </p>
                {showAllPlans && preselectedId && (
                    <button
                        onClick={() => setShowAllPlans(false)}
                        className="mt-4 text-sm text-blue-500 hover:underline flex items-center gap-1 mx-auto"
                    >
                        <ArrowLeft className="w-3 h-3" />
                        Back to selected plan
                    </button>
                )}
                {!isExpired && currentPlanTier > 0 && (
                    <div className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                        <Sparkles className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                        <span className="text-sm text-blue-700 dark:text-blue-300">
                            Select any plan to update your current subscription
                        </span>
                    </div>
                )}
            </div>

            {/* Plan Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {plans.map((plan) => (
                    <PricingCard
                        key={plan.id}
                        card={plan}
                        isDisabled={false}
                        isSelected={selectedPlan?.id === plan.id}
                        onSelect={handlePlanSelect}
                        actionType={getActionType(plan)}
                    />
                ))}
            </div>

            {/* Error */}
            {error && (
                <div className="max-w-lg mx-auto bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 text-center">
                    <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
                </div>
            )}
        </div>
    );
}

export default UnifiedPaymentFlow;
