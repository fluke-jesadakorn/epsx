'use client';

import { usePlanAccess } from '@/hooks/use-plan-access';
import { getPaymentReceiverAddress, getTokenAddress } from '@/lib/contracts/addresses';
import { supportedChains } from '@/shared/components/navigation/chain-selector';
import type { PricingCardData } from '@/shared/components/plans/pricing-card';
import { useCallback, useMemo } from 'react';
import { getAddress, parseUnits } from 'viem';
import { useAccount, useBalance, useChainId } from 'wagmi';
import type { UpgradePreviewData } from '../upgrade-banner';
import type { PaymentToken } from './use-payment-plans';

interface UsePaymentChainCtx {
    selectedToken: PaymentToken;
    selectedPlan: PricingCardData | null;
    effectivePreview: UpgradePreviewData | null;
}

export function usePaymentChain({ selectedToken, selectedPlan, effectivePreview }: UsePaymentChainCtx) {
    const { address } = useAccount();
    const chainId = useChainId();
    const { planAccess, loading: planAccessLoading, refetch: refetchPlanAccess } = usePlanAccess();

    const isChainSupported = useMemo(
        () => supportedChains.some(chain => chain.id === chainId),
        [chainId]
    );

    const tokenAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try { return getAddress(getTokenAddress(selectedToken.symbol, chainId)); }
        catch (_err) { return null; }
    }, [selectedToken, chainId, isChainSupported]);

    const receiverAddress = useMemo(() => {
        if (!isChainSupported) { return null; }
        try { return getAddress(getPaymentReceiverAddress(chainId)); }
        catch (_err) { return null; }
    }, [chainId, isChainSupported]);

    const amountInDecimals = useMemo(() => {
        if (!selectedPlan) { return 0n; }
        const rawAmt = effectivePreview?.amount_to_pay ?? selectedPlan.price;
        const raw = rawAmt.replace(/[^0-9.]/g, '');
        const parts = raw.split('.');
        const priceVal = parts.length > 1 ? `${parts[0]}.${parts.slice(1).join('')}` : raw;
        if (!priceVal || isNaN(Number(priceVal)) || Number(priceVal) <= 0) { return 0n; }
        return parseUnits(priceVal, selectedToken.decimals);
    }, [selectedPlan, selectedToken, effectivePreview]);

    const { data: balanceData } = useBalance({
        address,
        token: tokenAddress as `0x${string}`,
        chainId,
        query: { enabled: Boolean(address) && Boolean(tokenAddress) },
    });

    const currentPlanTier = useMemo(() => {
        const plan = planAccess?.plan_name;
        if (!plan) { return 0; }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const tierLevel = (planAccess as any)?.tier_level;
        return typeof tierLevel === 'number' ? tierLevel : 0;
    }, [planAccess]);

    const getActionType = useCallback((plan: PricingCardData) => {
        const planTier = typeof plan.tier_level === 'number' ? plan.tier_level : 0;
        if (planTier === currentPlanTier) { return 'extend'; }
        if (planTier > currentPlanTier) { return 'upgrade'; }
        if (planTier < currentPlanTier) { return 'downgrade'; }
        return 'select';
    }, [currentPlanTier]);

    return {
        address,
        isChainSupported,
        tokenAddress,
        receiverAddress,
        amountInDecimals,
        balanceData,
        planAccess,
        planAccessLoading,
        refetchPlanAccess,
        currentPlanTier,
        isExpired: planAccess?.status === 'expired',
        getActionType,
    };
}
