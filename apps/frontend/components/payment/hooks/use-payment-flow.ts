'use client';

import { getTransactionStatusAction, submitTransactionAction, switchPlanAction } from '@/app/actions/payments';
import { fmtAmt } from '@/shared/utils/formatting/currency';
import { getPublicPlansAction } from '@/app/actions/plans';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { getAddress, parseUnits } from 'viem';
import { useAccount, useBalance, useChainId } from 'wagmi';
import { usePlanAccess } from '@/hooks/use-plan-access';
import { useAuth } from '@/lib/auth';
import { getPaymentReceiverAddress, getTokenAddress } from '@/lib/contracts/addresses';
import { supportedChains } from '@/shared/components/navigation/chain-selector';
import { logger } from '@/shared/utils/logger';
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
    effective_price?: number;
    promotion_active?: boolean;
    promotion_discount?: number;
    promotion_ends_at?: string;
    currency?: string;
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

// Polling constants (module-level for stable references)
const MAX_POLL_MS = 5 * 60 * 1000;
const POLL_INTERVALS = [3000, 5000, 8000, 12000, 15000];

interface UsePaymentFlowContext {
    preselectedId?: string;
    initialPlans?: RawPlan[];
}

export function usePaymentFlow({ preselectedId, initialPlans = [] }: UsePaymentFlowContext) {
    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();

    // Capture URL params on mount (ref ensures stable initial values)
    const urlInitRef = useRef({
        step: searchParams.get('step'),
        planId: searchParams.get('planId'),
        token: searchParams.get('token'),
        tx: searchParams.get('tx'),
    });

    const { user, isLoading: isAuthLoading } = useAuth();
    const { address } = useAccount();
    const chainId = useChainId();
    const isAuthenticated = Boolean(user);

    const { planAccess, loading: planAccessLoading, refetch: refetchPlanAccess } = usePlanAccess();

    // Initialize step from URL (pay → confirm fallback since wallet can't resume)
    const [step, setStep] = useState<PaymentStep>(() => {
        const s = urlInitRef.current.step;
        if (s === 'confirm' || s === 'pay') { return 'confirm'; }
        if (s === 'verifying') { return 'verifying'; }
        if (s === 'success') { return 'success'; }
        // Skip plan selection when a plan is preselected (direct to confirm)
        if (urlInitRef.current.planId ?? preselectedId) { return 'confirm'; }
        return 'select';
    });
    const [plans, setPlans] = useState<PricingCardData[]>([]);
    const [selectedPlan, setSelectedPlan] = useState<PricingCardData | null>(null);
    // Initialize token from URL
    const [selectedToken, setSelectedToken] = useState<PaymentToken>(() => {
        const t = urlInitRef.current.token;
        if (t) {
            const found = PAYMENT_TOKENS.find(tk => tk.symbol === t);
            if (found) { return found; }
        }
        return PAYMENT_TOKENS[0];
    });
    const [loading, setLoading] = useState(initialPlans.length === 0);
    const [error, setError] = useState<string | null>(null);
    // Initialize txHash from URL
    const [txHash, setTxHash] = useState<string | null>(urlInitRef.current.tx ?? null);
    const [upgradePreview, setUpgradePreview] = useState<UpgradePreviewData | null>(null);
    const [showAllPlans, setShowAllPlans] = useState(false);

    // Fallback: build preview from planAccess data when API hasn't returned
    const effectivePreview = useMemo((): UpgradePreviewData | null => {
        if (upgradePreview) { return upgradePreview; }
        if (!planAccess || !selectedPlan) { return null; }
        const isSamePlan = planAccess.plan_id === String(selectedPlan.id);
        const credit = isSamePlan ? 0 : parseFloat(planAccess.proration_credit ?? '0');
        if (credit <= 0) { return null; }
        const newPrice = parseFloat(selectedPlan.price.replace(/[^0-9.]/g, ''));
        if (isNaN(newPrice) || newPrice <= 0) { return null; }
        const amountToPay = Math.max(0, newPrice - credit).toFixed(2);
        return {
            current_plan: null,
            new_plan: { id: String(selectedPlan.id), name: selectedPlan.title, price: newPrice.toString() },
            credit_from_current_plan: credit.toFixed(2),
            wallet_credit_balance: '0',
            total_credits_available: credit.toFixed(2),
            amount_to_pay: amountToPay,
            new_duration_days: 30,
            new_expiry_date: new Date(Date.now() + 30 * 86400000).toISOString(),
            is_upgrade_allowed: true,
        };
    }, [upgradePreview, planAccess, selectedPlan]);

    // --- URL sync: state → URL ---
    const lastUrlRef = useRef('');
    const selectedPlanId = selectedPlan?.id;

    useEffect(() => {
        const params = new URLSearchParams();

        if (step !== 'select') {
            // Store 'pay' as 'confirm' in URL (wallet interaction can't resume)
            params.set('step', step === 'pay' ? 'confirm' : step);
            if (selectedPlanId !== undefined && selectedPlanId !== null) {
                params.set('planId', String(selectedPlanId));
            }
            if (selectedToken.symbol !== 'USDT') {
                params.set('token', selectedToken.symbol);
            }
            if (txHash && (step === 'verifying' || step === 'success')) {
                params.set('tx', txHash);
            }
        }

        const qs = params.toString();
        const url = qs ? `${pathname}?${qs}` : pathname;

        if (url !== lastUrlRef.current) {
            lastUrlRef.current = url;
            router.replace(url, { scroll: false });
        }
    }, [step, selectedPlanId, selectedToken.symbol, txHash, pathname, router]);

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
        // Use backend-calculated reduced amount when available (upgrade pricing)
        const rawAmt = effectivePreview?.amount_to_pay ?? selectedPlan.price;
        const raw = rawAmt.replace(/[^0-9.]/g, '');
        const parts = raw.split('.');
        const priceVal = parts.length > 1
            ? `${parts[0]}.${parts.slice(1).join('')}`
            : raw;
        if (!priceVal || isNaN(Number(priceVal)) || Number(priceVal) <= 0) { return 0n; }
        return parseUnits(priceVal, selectedToken.decimals);
    }, [selectedPlan, selectedToken, effectivePreview]);

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
                const basePrice = typeof plan.current_price === 'string'
                    ? parseFloat(plan.current_price)
                    : plan.current_price;

                const hasPromo = plan.promotion_active === true &&
                    plan.effective_price !== undefined &&
                    plan.effective_price < basePrice;

                const displayPrice = hasPromo ? (plan.effective_price ?? basePrice) : basePrice;
                const isFree = displayPrice === 0;
                const currency = plan.currency ?? 'USD';
                const discount = plan.promotion_discount ?? 0;
                const savedAmt = hasPromo ? basePrice - displayPrice : 0;

                const parsedFeatures = (Array.isArray(plan.features)
                    ? plan.features
                    : typeof plan.features === 'string'
                        ? JSON.parse(plan.features)
                        : []) as Array<string | { text: string; included: boolean }>;

                return {
                    id: plan.id,
                    title: plan.name,
                    price: isFree ? 'Free' : `$${fmtAmt(displayPrice)} ${currency}`,
                    originalPrice: hasPromo ? `$${fmtAmt(basePrice)} ${currency}` : undefined,
                    features: parsedFeatures.map((f) => typeof f === 'string' ? { text: f, included: true } : f),
                    highlight: plan.is_highlighted ?? plan.is_promoted,
                    buttonText: isFree ? 'Start Free' : 'Select Plan',
                    promotions: hasPromo ? [`${Math.round(discount)}% OFF`] : [],
                    savings: hasPromo ? `Save $${fmtAmt(savedAmt)}` : undefined,
                    promotion_ends_at: hasPromo ? plan.promotion_ends_at : undefined,
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
                // URL planId takes priority over preselectedId prop
                const selectId = urlInitRef.current.planId ?? preselectedId;
                if (selectId) {
                    const preselected = transformed.find((p) => String(p.id) === String(selectId));
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

            // URL planId takes priority over preselectedId prop
            const selectId = urlInitRef.current.planId ?? preselectedId;
            if (selectId) {
                const preselected = transformed.find((p) => String(p.id) === String(selectId));
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

    const previewPlanIdRef = useRef<string | null>(null);

    useEffect(() => {
        if (step !== 'confirm' || !selectedPlan || !address) {
            return;
        }

        // Deduplicate: skip if already fetched for this plan
        if (previewPlanIdRef.current === String(selectedPlan.id)) {
            return;
        }
        previewPlanIdRef.current = String(selectedPlan.id);

        const fetchUpgradePreview = async () => {
            try {
                const baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
                const url = `${baseUrl}/api/payments/plans/upgrade_preview?new_plan_id=${selectedPlan.id}`;

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

    const isDowngrade = useMemo(() => {
        if (!selectedPlan || currentPlanTier === 0) { return false; }
        const planTier = typeof selectedPlan.tier_level === 'number' ? selectedPlan.tier_level : 0;
        return planTier < currentPlanTier && planTier !== currentPlanTier;
    }, [selectedPlan, currentPlanTier]);

    const handlePayment = () => {
        if (!selectedPlan) {
            setError('Please select a plan before proceeding');
            return;
        }
        if (isDowngrade) {
            setError('Downgrades are not available. You can only upgrade to a higher-tier plan.');
            return;
        }
        if (!address) {
            setError('Wallet not connected. Please connect your wallet.');
            return;
        }

        // Free upgrade: call switchPlan directly, skip blockchain
        if (amountInDecimals === 0n && effectivePreview && effectivePreview.current_plan) {
            setError(null);
            setStep('pay');
            const executeFreeUpgrade = async () => {
                try {
                    const result = await switchPlanAction(String(selectedPlan.id));
                    if (result.success) {
                        setStep('success');
                        refetchPlanAccess();
                    } else {
                        const msg = (result as { error?: { message?: string } }).error?.message ?? 'Upgrade failed';
                        setError(msg);
                        setStep('confirm');
                    }
                } catch (_err) {
                    setError('Upgrade failed. Please try again.');
                    setStep('confirm');
                }
            };
            void executeFreeUpgrade();
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
            const reqAmt = (effectivePreview?.amount_to_pay ?? selectedPlan.price).replace(/[^0-9.]/g, '');
            setError(`Insufficient ${selectedToken.symbol} balance. You have ${balanceData.formatted} ${selectedToken.symbol}, but ${reqAmt} is required.`);
            return;
        }

        setError(null);
        setStep('pay');
        transfer();
    };

    // --- Polling infrastructure ---
    const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const pollStartRef = useRef<number>(0);
    const pollAttemptRef = useRef<number>(0);
    const visCleanupRef = useRef<(() => void) | null>(null);

    const clearPoll = useCallback(() => {
        if (pollRef.current) {
            clearTimeout(pollRef.current);
            pollRef.current = null;
        }
    }, []);

    // Cleanup polling + visibility listener on unmount
    useEffect(() => () => {
        clearPoll();
        visCleanupRef.current?.();
    }, [clearPoll]);

    // Wallet disconnect detection during verification
    useEffect(() => {
        if (step !== 'verifying') { return; }
        if (address) { return; }
        // Wallet disconnected mid-verification
        clearPoll();
        setError('Wallet disconnected during verification. Your payment is safe — check your account later.');
    }, [step, address, clearPoll]);

    // Extracted polling logic (reusable for submit + URL resume)
    const startPolling = useCallback((hash: string) => {
        // Clean up previous visibility listener if any
        visCleanupRef.current?.();

        pollStartRef.current = Date.now();
        pollAttemptRef.current = 0;

        const schedulePoll = () => {
            const idx = Math.min(pollAttemptRef.current, POLL_INTERVALS.length - 1);
            const delay = POLL_INTERVALS[idx];

            pollRef.current = setTimeout(() => {
                void (async () => {
                    // Pause when tab is hidden (user switched to wallet app)
                    if (typeof document !== 'undefined' && document.hidden) {
                        schedulePoll();
                        return;
                    }

                    // Check timeout
                    if (Date.now() - pollStartRef.current > MAX_POLL_MS) {
                        clearPoll();
                        setError('Verification is taking longer than expected. Your payment is safe — check your account later or contact support.');
                        return;
                    }

                    try {
                        const status = await getTransactionStatusAction(hash);
                        if (status.success && status.data) {
                            const s = (status.data as { status: string }).status;
                            if (s === 'confirmed') {
                                clearPoll();
                                setStep('success');
                                refetchPlanAccess();
                                return;
                            } else if (s === 'failed' || s === 'expired') {
                                clearPoll();
                                setError('Payment verification failed. Please contact support.');
                                return;
                            }
                        }
                    } catch (_err) {
                        // 404 = not yet processed, keep polling
                        logger.error('[Payment] Poll status error:', _err);
                    }

                    pollAttemptRef.current += 1;
                    schedulePoll();
                })();
            }, delay);
        };

        // Resume polling when tab becomes visible again
        const handleVisibility = () => {
            if (!document.hidden && pollRef.current === null && pollStartRef.current > 0) {
                pollAttemptRef.current = 0; // Reset backoff on resume
                schedulePoll();
            }
        };
        if (typeof document !== 'undefined') {
            document.addEventListener('visibilitychange', handleVisibility);
            visCleanupRef.current = () => {
                document.removeEventListener('visibilitychange', handleVisibility);
            };
        }

        schedulePoll();
    }, [clearPoll, refetchPlanAccess]);

    // Resume polling on mount if URL has step=verifying + tx hash
    const hasResumedRef = useRef(false);

    useEffect(() => {
        if (hasResumedRef.current) { return; }
        if (step !== 'verifying' || !txHash) { return; }
        // Skip if polling already started (e.g., from submit effect)
        if (pollStartRef.current > 0) { return; }

        hasResumedRef.current = true;
        startPolling(txHash);
    }, [step, txHash, startPolling]);

    // Submit to backend when chain confirms, then poll for backend verification
    useEffect(() => {
        if (isConfirmed && transferTxHash && step === 'pay') {
            const submitPayment = async () => {
                try {
                    const rawAmt = effectivePreview?.amount_to_pay ?? selectedPlan?.price ?? '0';
                    const priceVal = rawAmt.replace(/[^0-9.]/g, '');

                    const result = await submitTransactionAction({
                        transaction_hash: transferTxHash,
                        plan_id: selectedPlan?.id as string,
                        expected_amount: priceVal,
                        currency: selectedToken.symbol,
                    });

                    if (result.success) {
                        setTxHash(transferTxHash);
                        setStep('verifying');
                        // Prevent resume effect from double-starting polling
                        hasResumedRef.current = true;
                        startPolling(transferTxHash);
                    } else {
                        setError(result.error?.message ?? 'Payment submitted but verification pending');
                    }
                } catch (_err) {
                    setError('Payment confirmed but backend submission failed');
                }
            };

            void submitPayment();
        }
    }, [isConfirmed, transferTxHash, step, selectedPlan, selectedToken, startPolling]);

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
        upgradePreview: effectivePreview,
        showAllPlans,
        setShowAllPlans,
        isChainSupported,
        tokenAddress,
        balanceData,
        receiverAddress,
        amountInDecimals,
        isTransferring,
        isConfirming,
        isConfirmed,
        currentPlanTier,
        isExpired,
        isDowngrade,
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
