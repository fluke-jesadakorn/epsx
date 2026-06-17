'use client';

import { useState } from 'react';
import { useAccount } from 'wagmi';
import { useAuth } from '@/lib/auth';
import { cn } from '@/lib/utils';
import { ChainVerificationCard } from './chain-verification-card';
import { CurrentAccessCard } from './current-access-card';
import { usePaymentFlow, PAYMENT_TOKENS } from './hooks/use-payment-flow';
import {
    LoadingStep,
    SignInStep,
    ErrorStep,
    VerifyingStep,
    SuccessStep,
    ConfirmStep,
    PlanGridView
} from './payment-flow-steps';

type PaymentType = 'plan' | 'access-plan' | 'permission';

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

interface UnifiedPaymentFlowProps {
    paymentType: PaymentType;
    preselectedId?: string;
    title?: string;
    description?: string;
    className?: string;
    initialPlans?: RawPlan[];
}

export function UnifiedPaymentFlow({
    paymentType,
    preselectedId,
    title,
    description,
    className,
    initialPlans = [],
}: UnifiedPaymentFlowProps) {
    const { openSignInModal } = useAuth();
    const { isConnected, connector } = useAccount();
    const [isSwitchingChain, setIsSwitchingChain] = useState(false);
    const [isAddingChain, setIsAddingChain] = useState(false);

    const {
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
        balanceData,
        amountInDecimals,
        isTransferring,
        isConfirming,
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
    } = usePaymentFlow({ preselectedId, initialPlans });

    const handleSwitchToBsc = async () => {
        if (!connector) { return; }
        setIsSwitchingChain(true);
        try {
            const provider = await connector.getProvider() as { request: (args: { method: string; params: unknown[] }) => Promise<unknown> };
            await provider.request({
                method: 'wallet_switchEthereumChain',
                params: [{ chainId: '0x38' }],
            });
        } catch (err: unknown) {
            const e = err as { code?: number };
            if (e?.code === 4902) {
                try {
                    const provider = await connector.getProvider() as { request: (args: { method: string; params: unknown[] }) => Promise<unknown> };
                    await provider.request({
                        method: 'wallet_addEthereumChain',
                        params: [{
                            chainId: '0x38',
                            chainName: 'BNB Smart Chain',
                            nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 },
                            rpcUrls: ['https://bsc-dataseed.binance.org/'],
                            blockExplorerUrls: ['https://bscscan.com'],
                        }],
                    });
                } catch (_addErr) {
                    // user rejected adding network
                }
            }
        } finally {
            setIsSwitchingChain(false);
        }
    };

    const handleAddBscChain = async () => {
        if (!connector) { return; }
        setIsAddingChain(true);
        try {
            const provider = await connector.getProvider() as { request: (args: { method: string; params: unknown[] }) => Promise<unknown> };
            await provider.request({
                method: 'wallet_addEthereumChain',
                params: [{
                    chainId: '0x38',
                    chainName: 'BNB Smart Chain Mainnet',
                    nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 },
                    rpcUrls: ['https://bsc-dataseed.binance.org/'],
                    blockExplorerUrls: ['https://bscscan.com'],
                }],
            });
        } catch (_err) {
            // user rejected
        } finally {
            setIsAddingChain(false);
        }
    };

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

    if (isAuthLoading || (isAuthenticated && (loading || planAccessLoading))) {
        return <LoadingStep className={className} />;
    }

    if (!isAuthenticated) {
        return <SignInStep isConnected={isConnected} onSignIn={openSignInModal} className={className} />;
    }

    if (isConnected && !isChainSupported) {
        return <ChainVerificationCard className={className} />;
    }

    if (error && plans.length === 0) {
        return <ErrorStep error={error} onRetry={fetchPlans} className={className} />;
    }

    if (step === 'verifying') {
        return <VerifyingStep planTitle={selectedPlan?.title ?? ''} txHash={txHash} errorMessage={error} className={className} />;
    }

    if (step === 'success') {
        return <SuccessStep planTitle={selectedPlan?.title ?? ''} txHash={txHash} className={className} />;
    }

    if ((step === 'confirm' || step === 'pay') && selectedPlan) {
        return (
            <ConfirmStep
                selectedPlan={selectedPlan}
                selectedToken={selectedToken}
                paymentTokens={PAYMENT_TOKENS}
                balanceData={balanceData}
                amountInDecimals={amountInDecimals}
                error={error}
                isTransferring={isTransferring}
                isConfirming={isConfirming}
                isChainSupported={isChainSupported}
                onSwitchChain={handleSwitchToBsc}
                onAddChain={handleAddBscChain}
                isSwitchingChain={isSwitchingChain}
                isAddingChain={isAddingChain}
                planAccess={planAccess}
                plans={plans}
                upgradePreview={upgradePreview}
                isDowngrade={isDowngrade}
                onBack={() => setStep('select')}
                onTokenSelect={setSelectedToken}
                onPayment={handlePayment}
                className={className}
            />
        );
    }

    return (
        <div className={cn('space-y-8', className)}>
            <CurrentAccessCard paymentType={paymentType} />

            <div className="text-center">
                <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-2">
                    {getPageTitle()}
                </h2>
                <p className="text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
                    {getPageDescription()}
                </p>
            </div>

            <PlanGridView
                plans={plans}
                selectedPlan={selectedPlan}
                currentPlanTier={currentPlanTier}
                isExpired={isExpired}
                error={error}
                preselectedId={preselectedId}
                showAllPlans={showAllPlans}
                getActionType={getActionType}
                onPlanSelect={handlePlanSelect}
                onBackToSelected={() => setShowAllPlans(false)}
                className=""
            />
        </div>
    );
}

export default UnifiedPaymentFlow;
