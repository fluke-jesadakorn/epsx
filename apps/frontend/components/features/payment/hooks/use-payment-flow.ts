'use client';

import { getTransactionStatusAction, submitTransactionAction } from '@/app/actions/payments';
import { getPublicPlansAction } from '@/app/actions/plans';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { getAddress, parseUnits } from 'viem';
import { useAccount, useBalance, useChainId } from 'wagmi';
import { usePlanAccess } from '@/hooks/use-plan-access';
import { useAuth } from '@/lib/auth';
import { getPaymentReceiverAddress, getTokenAddress } from '@/lib/contracts/addresses';
import { supportedChains } from '@/shared/components/navigation/chain-selector';
import { logger } from '@/shared/utils/logger';
import { useAddTokenToWallet } from './use-add-token-to-wallet';
import { useDirectTokenTransfer } from './use-direct-token-transfer';
import type { PricingCardData } from '@/shared/components/plans/pricing-card';
import type { UpgradePreviewData } from '../upgrade-banner';

type PaymentStep = 'select' | 'confirm' | 'pay' | 'verifying' | 'success';

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

interface PaymentToken {
    symbol: string;
    name: string;
    decimals: number;
}

export const PAYMENT_TOKENS: PaymentToken[] = [
    { symbol: 'USDT', name: 'Tether USD', decimals: 18 },
    { symbol: 'USDC', name: 'USD Coin', decimals: 18 },
    { symbol: 'DAI', name: 'Dai Stablecoin', decimals: 18 },
];

interface UsePaymentFlowContext {
    preselectedId?: string;
    initialPlans?: RawPlan[];
}

export function usePaymentFlow({ preselectedId, initialPlans = [] }: UsePaymentFlowContext) {
    const { user, isLoading: isAuthLoading } = useAuth();
    const { address } = useAccount();
    const chainId = useChainId();
    const isAuthenticated = Boolean(user);

    const { planAccess, loading: planAccessLoading, refetch: refetchPlanAccess } = usePlanAccess();

    const [step, setStep] = useState<PaymentStep>('select');
    const [plans, setPlans] = useState<PricingCardData[]>([]);
    const [selectedPlan, setSelectedPlan] = useState<PricingCardData | null>(null);
    const [selectedToken, setSelectedToken] = useState<PaymentToken>(PAYMENT_TOKENS[0]);
    const [loading, setLoading] = useState(initialPlans.length === 0);
    const [error, setError] = useState<string | null>(null);
    const [txHash, setTxHash] = useState<string | null>(null);
    const [upgradePreview, setUpgradePreview] = useState<UpgradePreviewData | null>(null);
    const [showAllPlans, setShowAllPlans] = useState(false);

    const isChainSupported = useMemo(
        () => supportedChains.some(chain => chain.id === chainId),
        [chainId]
    );

    const tokenAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try {
            return getAddress(getTokenAddress(selectedToken.symbol, chainId));
        } catch (_err) {
            return null;
        }
    }, [selectedToken, chainId, isChainSupported]);

    const { data: balanceData } = useBalance({
        address,
        token: tokenAddress as `0x${string}`,
        chainId,
        query: { enabled: Boolean(address) && Boolean(tokenAddress) },
    });

    const receiverAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try {
            return getAddress(getPaymentReceiverAddress(chainId));
        } catch (_err) {
            return null;
        }
    }, [chainId, isChainSupported]);

    const amountInDecimals = useMemo(() => {
        if (!selectedPlan) { return 0n; }
        const priceVal = selectedPlan.price.replace(/[^0-9.]/g, '');
        return parseUnits(priceVal, selectedToken.decimals);
    }, [selectedPlan, selectedToken]);

    const { addToken, isAdding: isAddingToken, isTokenAdded } = useAddTokenToWallet();

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
            setStep('confirm');
        }
    });

    const currentPlanTier = useMemo(() => {
        const plan = planAccess?.plan_name;
        if (!plan) { return 0; }
        const tierLevel = (planAccess as any)?.tier_level;
        return typeof tierLevel === 'number' ? tierLevel : 0;
    }, [planAccess]);

    const isExpired = planAccess?.status === 'expired';

    const getActionType = useCallback((plan: PricingCardData) => {
        const planTier = typeof plan.tier_level === 'number' ? plan.tier_level : 0;
        if (planTier === currentPlanTier) { return 'extend'; }
        if (planTier > currentPlanTier) { return 'upgrade'; }
        if (planTier < currentPlanTier) { return 'downgrade'; }
        return 'select';
    }, [currentPlanTier]);

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

    const fetchPlans = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const result = await getPublicPlansAction();

            if (result.success && result.data && Array.isArray(result.data)) {
                const transformed = transformPlans(result.data);
                setPlans(transformed);
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

    useEffect(() => {
        if (initialPlans.length > 0) {
            const transformed = transformPlans(initialPlans);
            setPlans(transformed);
            setLoading(false);

            if (preselectedId) {
                const preselected = transformed.find((p) => String(p.id) === String(preselectedId));
                if (preselected) {
                    setSelectedPlan(preselected);
                }
            }
        } else {
            void fetchPlans();
        }
    }, [initialPlans, preselectedId, transformPlans, fetchPlans]);

    useEffect(() => {
        const timeout = setTimeout(() => {
            if (loading) {
                setError('Loading timeout - please refresh the page');
                setLoading(false);
            }
        }, 10000);

        return () => clearTimeout(timeout);
    }, [loading]);

    useEffect(() => {
        if (step !== 'confirm' || !selectedPlan || !address) {
            return;
        }

        const fetchUpgradePreview = async () => {
            try {
                const baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
                const url = `${baseUrl}/api/payments/subscriptions/upgrade-preview?new_plan_id=${selectedPlan.id}`;

                const response = await fetch(url, {
                    method: 'GET',
                    headers: {
                        Accept: 'application/json',
                        'Content-Type': 'application/json',
                    },
                    credentials: 'include',
                });

                if (response.ok) {
                    const result: { success?: boolean; data?: UpgradePreviewData } = await response.json();
                    if ((result.success ?? false) && result.data) {
                        setUpgradePreview(result.data);
                    }
                }
            } catch (_err) {
                logger.error('Failed to fetch upgrade preview', _err);
            }
        };

        void fetchUpgradePreview();
    }, [step, selectedPlan, address]);

    const handlePlanSelect = (plan: PricingCardData) => {
        setSelectedPlan(plan);
        setError(null);
        setStep('confirm');
    };

    const handlePayment = async () => {
        if (!selectedPlan) {
            setError('Please select a plan before proceeding');
            return;
        }
        if (!address) {
            setError('Wallet not connected. Please connect your wallet.');
            return;
        }
        if (!receiverAddress) {
            setError('Payment not available on this network. Please switch to BSC.');
            return;
        }
        if (!tokenAddress) {
            setError(`${selectedToken.symbol} not available on this network. Please switch to BSC.`);
            return;
        }

        if (balanceData && balanceData.value < amountInDecimals) {
            setError(`Insufficient ${selectedToken.symbol} balance. You have ${balanceData.formatted} ${selectedToken.symbol}, but ${selectedPlan.price.replace(/[^0-9.]/g, '')} is required.`);
            return;
        }

        setError(null);
        setStep('pay');

        if (!isTokenAdded(selectedToken.symbol)) {
            await addToken(selectedToken.symbol);
        }

        transfer();
    };

    const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

    // Cleanup polling on unmount
    useEffect(() => {
        return () => {
            if (pollRef.current) { clearInterval(pollRef.current); }
        };
    }, []);

    // Submit to backend when chain confirms, then poll for backend verification
    useEffect(() => {
        if (isConfirmed && transferTxHash && step === 'pay') {
            const submitPayment = async () => {
                try {
                    const priceVal = parseFloat(selectedPlan?.price.replace(/[^0-9.]/g, '') ?? '0');

                    const result = await submitTransactionAction({
                        transaction_hash: transferTxHash,
                        plan_id: selectedPlan?.id as string,
                        expected_amount: priceVal,
                        currency: selectedToken.symbol,
                    });

                    if (result.success) {
                        setTxHash(transferTxHash);
                        setStep('verifying');

                        // Poll backend for confirmation
                        pollRef.current = setInterval(() => {
                            void (async () => {
                                try {
                                    const status = await getTransactionStatusAction(transferTxHash);
                                    if (status.success && status.data) {
                                        const s = (status.data as { status: string }).status;
                                        if (s === 'confirmed') {
                                            if (pollRef.current) { clearInterval(pollRef.current); }
                                            setStep('success');
                                            refetchPlanAccess();
                                        } else if (s === 'failed') {
                                            if (pollRef.current) { clearInterval(pollRef.current); }
                                            setError('Payment verification failed. Please contact support.');
                                        }
                                    }
                                } catch (_err) {
                                    // 404 = not yet processed, keep polling
                                    logger.error('[Payment] Poll status error:', _err);
                                }
                            })();
                        }, 3000);
                    } else {
                        setError(result.error?.message ?? 'Payment submitted but verification pending');
                    }
                } catch (_err) {
                    setError('Payment confirmed but backend submission failed');
                }
            };

            void submitPayment();
        }
    }, [isConfirmed, transferTxHash, step, selectedPlan, selectedToken, chainId, refetchPlanAccess]);

    return {
        step,
        setStep,
        plans,
        selectedPlan,
        selectedToken,
        setSelectedToken,
        loading,
        error,
        txHash,
        upgradePreview,
        showAllPlans,
        setShowAllPlans,
        isChainSupported,
        tokenAddress,
        balanceData,
        receiverAddress,
        amountInDecimals,
        isAddingToken,
        isTokenAdded,
        isTransferring,
        isConfirming,
        isConfirmed,
        currentPlanTier,
        isExpired,
        planAccess,
        planAccessLoading,
        isAuthenticated,
        isAuthLoading,
        getActionType,
        handlePlanSelect,
        handlePayment,
        fetchPlans,
    };
}
