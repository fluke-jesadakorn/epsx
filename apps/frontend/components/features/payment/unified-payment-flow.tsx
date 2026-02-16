'use client';

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
    SuccessStep,
    ConfirmStep,
    SinglePlanView,
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
    const { isConnected } = useAccount();

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
        isAddingToken,
        isTokenAdded,
        isTransferring,
        isConfirming,
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
    } = usePaymentFlow({ preselectedId, initialPlans });

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

    if (step === 'success') {
        return <SuccessStep planTitle={selectedPlan?.title ?? ''} txHash={txHash} className={className} />;
    }

    if (step === 'confirm' && selectedPlan) {
        return (
            <ConfirmStep
                selectedPlan={selectedPlan}
                selectedToken={selectedToken}
                paymentTokens={PAYMENT_TOKENS}
                balanceData={balanceData}
                amountInDecimals={amountInDecimals}
                error={error}
                isAddingToken={isAddingToken}
                isTransferring={isTransferring}
                isConfirming={isConfirming}
                isTokenAdded={isTokenAdded}
                planAccess={planAccess}
                plans={plans}
                upgradePreview={upgradePreview}
                onBack={() => setStep('select')}
                onTokenSelect={setSelectedToken}
                onPayment={handlePayment}
                className={className}
            />
        );
    }

    if (preselectedId && selectedPlan && !showAllPlans && step === 'select') {
        return (
            <SinglePlanView
                selectedPlan={selectedPlan}
                onPlanSelect={handlePlanSelect}
                onShowAllPlans={() => setShowAllPlans(true)}
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
