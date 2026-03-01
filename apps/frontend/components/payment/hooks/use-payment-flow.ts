'use client';

import { submitTransactionAction, switchPlanAction } from '@/app/actions/payments';
import { useAuth } from '@/lib/auth';
import { logger } from '@/shared/utils/logger';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import type { UpgradePreviewData } from '../upgrade-banner';
import { useDirectTokenTransfer } from './use-direct-token-transfer';
import { usePaymentChain } from './use-payment-chain';
import { usePaymentPlans, type RawPlan } from './use-payment-plans';
import { usePaymentPolling } from './use-payment-polling';
import type { PricingCardData } from '@/shared/components/plans/pricing-card';

export { PAYMENT_TOKENS } from './use-payment-plans';

export type PaymentStep = 'select' | 'confirm' | 'pay' | 'verifying' | 'success';

interface UsePaymentFlowContext {
    preselectedId?: string;
    initialPlans?: RawPlan[];
}

export function usePaymentFlow({ preselectedId, initialPlans = [] }: UsePaymentFlowContext) {
    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();

    const urlInitRef = useRef({
        step: searchParams.get('step'),
        planId: searchParams.get('planId'),
        token: searchParams.get('token'),
        tx: searchParams.get('tx'),
    });

    const { user, isLoading: isAuthLoading } = useAuth();
    const isAuthenticated = Boolean(user);

    const [step, setStep] = useState<PaymentStep>(() => {
        const s = urlInitRef.current.step;
        if (s === 'confirm' || s === 'pay') { return 'confirm'; }
        if (s === 'verifying') { return 'verifying'; }
        if (s === 'success') { return 'success'; }
        if (urlInitRef.current.planId ?? preselectedId) { return 'confirm'; }
        return 'select';
    });

    const [txHash, setTxHash] = useState<string | null>(urlInitRef.current.tx ?? null);
    const [upgradePreview, setUpgradePreview] = useState<UpgradePreviewData | null>(null);

    const planState = usePaymentPlans({
        preselectedId,
        initialPlans,
        urlPlanId: urlInitRef.current.planId,
        urlToken: urlInitRef.current.token,
    });
    const { selectedPlan, setSelectedPlan, selectedToken, error, setError } = planState;

    const effectivePreview = useMemo((): UpgradePreviewData | null => {
        if (upgradePreview) { return upgradePreview; }
        if (!planState.plans.length || !selectedPlan) { return null; }
        return null; // planAccess-based fallback computed after chain hook
    }, [upgradePreview, planState.plans.length, selectedPlan]);

    const chain = usePaymentChain({ selectedToken, selectedPlan, effectivePreview });
    const { planAccess, refetchPlanAccess, currentPlanTier } = chain;

    // Rebuild effectivePreview with planAccess available
    const fullEffectivePreview = useMemo((): UpgradePreviewData | null => {
        if (upgradePreview) { return upgradePreview; }
        if (!planAccess || !selectedPlan) { return null; }
        const isSamePlan = planAccess.plan_id === String(selectedPlan.id);
        const credit = isSamePlan ? 0 : parseFloat(planAccess.proration_credit ?? '0');
        if (credit <= 0) { return null; }
        const newPrice = parseFloat(selectedPlan.price.replace(/[^0-9.]/g, ''));
        if (isNaN(newPrice) || newPrice <= 0) { return null; }
        return {
            current_plan: null,
            new_plan: { id: String(selectedPlan.id), name: selectedPlan.title, price: newPrice.toString() },
            credit_from_current_plan: credit.toFixed(2),
            wallet_credit_balance: '0',
            total_credits_available: credit.toFixed(2),
            amount_to_pay: Math.max(0, newPrice - credit).toFixed(2),
            new_duration_days: 30,
            new_expiry_date: new Date(Date.now() + 30 * 86400000).toISOString(),
            is_upgrade_allowed: true,
        };
    }, [upgradePreview, planAccess, selectedPlan]);

    const { transfer, txHash: transferTxHash, isTransferring, isConfirming, isConfirmed } = useDirectTokenTransfer({
        tokenAddress: chain.tokenAddress,
        receiverAddress: chain.receiverAddress,
         
        amount: chain.amountInDecimals,
        onError: (msg: any) => {
            setError(typeof msg === 'string' ? msg : 'An error occurred');
            setStep('confirm');
        },
    });

    const polling = usePaymentPolling({ step, address: chain.address, txHash, refetchPlanAccess, setStep, setError });

    // URL sync
    const lastUrlRef = useRef('');
    const selectedPlanId = selectedPlan?.id;
    useEffect(() => {
        const params = new URLSearchParams();
        if (step !== 'select') {
            params.set('step', step === 'pay' ? 'confirm' : step);
            if (selectedPlanId !== undefined && selectedPlanId !== null) { params.set('planId', String(selectedPlanId)); }
            if (selectedToken.symbol !== 'USDT') { params.set('token', selectedToken.symbol); }
            if (txHash && (step === 'verifying' || step === 'success')) { params.set('tx', txHash); }
        }
        const qs = params.toString();
        const url = qs ? `${pathname}?${qs}` : pathname;
        if (url !== lastUrlRef.current) { lastUrlRef.current = url; router.replace(url, { scroll: false }); }
    }, [step, selectedPlanId, selectedToken.symbol, txHash, pathname, router]);

    // Upgrade preview fetch
    const previewPlanIdRef = useRef<string | null>(null);
    useEffect(() => {
        if (step !== 'confirm' || !selectedPlan || !chain.address) { return; }
        if (previewPlanIdRef.current === String(selectedPlan.id)) { return; }
        previewPlanIdRef.current = String(selectedPlan.id);
        void (async () => {
            try {
                const baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
                const res = await fetch(`${baseUrl}/api/payments/plans/upgrade_preview?new_plan_id=${selectedPlan.id}`, {
                    method: 'GET', headers: { Accept: 'application/json', 'Content-Type': 'application/json' }, credentials: 'include',
                });
                if (res.ok) {
                    const result: { success?: boolean; data?: UpgradePreviewData } = await res.json();
                    if ((result.success ?? false) && result.data) { setUpgradePreview(result.data); }
                }
            } catch (_err) { logger.error('Failed to fetch upgrade preview', _err); }
        })();
    }, [step, selectedPlan, chain.address]);

    const isDowngrade = useMemo(() => {
        if (!selectedPlan || currentPlanTier === 0) { return false; }
        const planTier = typeof selectedPlan.tier_level === 'number' ? selectedPlan.tier_level : 0;
        return planTier < currentPlanTier && planTier !== currentPlanTier;
    }, [selectedPlan, currentPlanTier]);

    const handlePlanSelect = (plan: PricingCardData) => {
        setSelectedPlan(plan);
        setError(null);
        setStep('confirm');
    };

    const handlePayment = useCallback(() => {
        if (!selectedPlan) { setError('Please select a plan before proceeding'); return; }
        if (isDowngrade) { setError('Downgrades are not available. You can only upgrade to a higher-tier plan.'); return; }
        if (!chain.address) { setError('Wallet not connected. Please connect your wallet.'); return; }
        if (chain.amountInDecimals === 0n && fullEffectivePreview?.current_plan) {
            setError(null);
            setStep('pay');
            void (async () => {
                try {
                    const result = await switchPlanAction(String(selectedPlan.id));
                    if (result.success) { setStep('success'); refetchPlanAccess(); }
                    else { setError((result as { error?: { message?: string } }).error?.message ?? 'Upgrade failed'); setStep('confirm'); }
                } catch (_err) { setError('Upgrade failed. Please try again.'); setStep('confirm'); }
            })();
            return;
        }
        if (!chain.receiverAddress) { setError('Payment not available on this network. Please switch to BSC.'); return; }
        if (!chain.tokenAddress) { setError(`${selectedToken.symbol} not available on this network. Please switch to BSC.`); return; }
        if (chain.balanceData && chain.balanceData.value < chain.amountInDecimals) {
            const reqAmt = (fullEffectivePreview?.amount_to_pay ?? selectedPlan.price).replace(/[^0-9.]/g, '');
            setError(`Insufficient ${selectedToken.symbol} balance. You have ${chain.balanceData.formatted} ${selectedToken.symbol}, but ${reqAmt} is required.`);
            return;
        }
        setError(null);
        setStep('pay');
        transfer();
    }, [selectedPlan, isDowngrade, chain, fullEffectivePreview, selectedToken, setError, refetchPlanAccess, transfer]);

    // Submit to backend when chain confirms
    useEffect(() => {
        if (!isConfirmed || !transferTxHash || step !== 'pay') { return; }
        void (async () => {
            try {
                const rawAmt = fullEffectivePreview?.amount_to_pay ?? selectedPlan?.price ?? '0';
                const result = await submitTransactionAction({
                    transaction_hash: transferTxHash,
                    plan_id: selectedPlan?.id as string,
                    expected_amount: rawAmt.replace(/[^0-9.]/g, ''),
                    currency: selectedToken.symbol,
                });
                if (result.success) {
                    setTxHash(transferTxHash);
                    setStep('verifying');
                    polling.hasResumedRef.current = true;
                    polling.startPolling(transferTxHash);
                } else {
                    setError(result.error?.message ?? 'Payment submitted but verification pending');
                }
            } catch (_err) { setError('Payment confirmed but backend submission failed'); }
        })();
    }, [isConfirmed, transferTxHash, step, selectedPlan, selectedToken, fullEffectivePreview, polling, setError]);

    return {
        step, setStep,
        ...planState,
        error, setError,
        txHash,
        upgradePreview: fullEffectivePreview,
        ...chain,
        isTransferring, isConfirming, isConfirmed,
        isDowngrade,
        isAuthenticated, isAuthLoading,
        handlePlanSelect, handlePayment,
        fetchPlans: planState.fetchPlans,
    };
}
