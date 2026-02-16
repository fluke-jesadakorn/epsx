'use client';

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
import { cn } from '@/lib/utils';
import { PlanComparisonCard } from './plan-comparison-card';
import { PricingCard } from '@/shared/components/plans/pricing-card';
import type { PricingCardData } from '@/shared/components/plans/pricing-card';
import type { PlanAccessData } from '@/shared/types/payment';
import type { UpgradePreviewData } from './upgrade-banner';
import type { PAYMENT_TOKENS } from './hooks/use-payment-flow';

interface PaymentToken {
    symbol: string;
    name: string;
    decimals: number;
}

interface StepIndicatorProps {
    currentStep: 'select' | 'confirm' | 'pay' | 'success';
}

export function StepIndicator({ currentStep }: StepIndicatorProps) {
    const steps = [
        { id: 'select', label: 'Select Plan' },
        { id: 'confirm', label: 'Confirm' },
        { id: 'pay', label: 'Payment' },
        { id: 'success', label: 'Complete' }
    ];

    const currentIndex = steps.findIndex(s => s.id === currentStep);

    return (
        <div className="flex items-center justify-center gap-2 mb-8">
            {steps.map((stepItem, idx) => (
                <div key={stepItem.id} className="flex items-center gap-2">
                    <div
                        className={cn(
                            'flex items-center justify-center w-8 h-8 rounded-full text-sm font-bold transition-all',
                            idx <= currentIndex
                                ? 'bg-blue-600 text-white'
                                : 'bg-gray-200 dark:bg-gray-700 text-gray-500'
                        )}
                    >
                        {idx + 1}
                    </div>
                    <span
                        className={cn(
                            'text-sm font-medium transition-colors',
                            idx <= currentIndex
                                ? 'text-blue-600 dark:text-blue-400'
                                : 'text-gray-500'
                        )}
                    >
                        {stepItem.label}
                    </span>
                    {idx < steps.length - 1 && (
                        <div
                            className={cn(
                                'w-12 h-0.5 mx-2 transition-colors',
                                idx < currentIndex
                                    ? 'bg-blue-600'
                                    : 'bg-gray-200 dark:bg-gray-700'
                            )}
                        />
                    )}
                </div>
            ))}
        </div>
    );
}

interface LoadingStepProps {
    className?: string;
}

export function LoadingStep({ className }: LoadingStepProps) {
    return (
        <div className={cn('py-12', className)}>
            <div className="flex flex-col items-center justify-center gap-4">
                <Loader2 className="w-12 h-12 text-blue-500 animate-spin" />
                <p className="text-gray-600 dark:text-gray-400">Loading payment options...</p>
            </div>
        </div>
    );
}

interface SignInStepProps {
    isConnected: boolean;
    onSignIn: () => void;
    className?: string;
}

export function SignInStep({ isConnected, onSignIn, className }: SignInStepProps) {
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
                    onClick={onSignIn}
                    className="w-full py-3.5 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-blue-900/20 active:scale-[0.98]"
                >
                    {isConnected ? 'Sign In' : 'Connect Wallet'}
                </button>
            </div>
        </div>
    );
}

interface ErrorStepProps {
    error: string;
    onRetry: () => void;
    className?: string;
}

export function ErrorStep({ error, onRetry, className }: ErrorStepProps) {
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
                    onClick={onRetry}
                    className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                    Try Again
                </button>
            </div>
        </div>
    );
}

interface SuccessStepProps {
    planTitle: string;
    txHash: string | null;
    className?: string;
}

export function SuccessStep({ planTitle, txHash, className }: SuccessStepProps) {
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
                    Your {planTitle} plan is now active.
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

interface ConfirmStepProps {
    selectedPlan: PricingCardData;
    selectedToken: PaymentToken;
    paymentTokens: typeof PAYMENT_TOKENS;
    balanceData: { value: bigint; formatted: string } | undefined;
    amountInDecimals: bigint;
    error: string | null;
    isAddingToken: boolean;
    isTransferring: boolean;
    isConfirming: boolean;
    isTokenAdded: (symbol: string) => boolean;
    planAccess: PlanAccessData | null;
    plans: PricingCardData[];
    upgradePreview: UpgradePreviewData | null;
    onBack: () => void;
    onTokenSelect: (token: PaymentToken) => void;
    onPayment: () => void;
    className?: string;
}

export function ConfirmStep({
    selectedPlan,
    selectedToken,
    paymentTokens,
    balanceData,
    amountInDecimals,
    error,
    isAddingToken,
    isTransferring,
    isConfirming,
    isTokenAdded,
    planAccess,
    plans,
    upgradePreview,
    onBack,
    onTokenSelect,
    onPayment,
    className
}: ConfirmStepProps) {
    return (
        <div className={cn('max-w-4xl mx-auto', className)}>
            <button
                onClick={onBack}
                className="flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 mb-6 transition-colors"
            >
                <ArrowLeft className="w-4 h-4" />
                Back to plans
            </button>

            {planAccess && (
                <div className="mb-8">
                    <PlanComparisonCard
                        currentPlan={
                            planAccess.status !== 'no_plan'
                                ? {
                                      name: planAccess.plan_name ?? 'Unknown Plan',
                                      tier_level: planAccess.tier_level ?? 0,
                                      expires_at: planAccess.plan_expires_at,
                                      days_remaining: planAccess.days_remaining,
                                      status: planAccess.status,
                                      features: plans.find(p => p.tier_level === (planAccess.tier_level ?? 0))?.features.map(f => f.text),
                                      price: plans.find(p => p.tier_level === (planAccess.tier_level ?? 0))?.price ? parseFloat(plans.find(p => p.tier_level === (planAccess.tier_level ?? 0))!.price.replace(/[^0-9.]/g, '')) : undefined,
                                  }
                                : null
                        }
                        newPlan={{
                            id: selectedPlan.id.toString(),
                            name: selectedPlan.title,
                            tier_level: selectedPlan.tier_level ?? 0,
                            price: parseFloat(selectedPlan.price.replace(/[^0-9.]/g, '')),
                            duration_days: 30,
                            features: selectedPlan.features.map((f) => f.text),
                        }}
                        upgradePreview={
                            upgradePreview
                                ? {
                                      credit_amount: parseFloat(upgradePreview.upgrade_details.remaining_credit),
                                      bonus_days: upgradePreview.upgrade_details.bonus_days,
                                      new_expiry_date: upgradePreview.upgrade_details.new_expiry_date,
                                  }
                                : null
                        }
                    />
                </div>
            )}

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-8 shadow-xl border border-blue-200/50 dark:border-blue-700/50">
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
                    Confirm Your Order
                </h2>

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

                <div className="mb-6">
                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Pay with
                    </p>
                    <div className="flex gap-2">
                        {paymentTokens.map((token) => (
                            <button
                                key={token.symbol}
                                onClick={() => onTokenSelect(token)}
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

                {error && (
                    <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 mb-6">
                        <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
                    </div>
                )}

                <button
                    onClick={onPayment}
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

interface SinglePlanViewProps {
    selectedPlan: PricingCardData;
    onPlanSelect: (plan: PricingCardData) => void;
    onShowAllPlans: () => void;
    className?: string;
}

export function SinglePlanView({ selectedPlan, onPlanSelect, onShowAllPlans, className }: SinglePlanViewProps) {
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
                    onSelect={onPlanSelect}
                    isSelected={true}
                    actionType="select"
                    isDisabled={false}
                />

                <div className="text-center mt-8">
                    <button
                        onClick={onShowAllPlans}
                        className="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-500 underline transition-colors"
                    >
                        View other plans
                    </button>
                </div>
            </div>
        </div>
    );
}

interface PlanGridViewProps {
    plans: PricingCardData[];
    selectedPlan: PricingCardData | null;
    currentPlanTier: number;
    isExpired: boolean;
    error: string | null;
    preselectedId?: string;
    showAllPlans: boolean;
    getActionType: (plan: PricingCardData) => 'select' | 'upgrade' | 'downgrade' | 'extend';
    onPlanSelect: (plan: PricingCardData) => void;
    onBackToSelected: () => void;
    className?: string;
}

export function PlanGridView({
    plans,
    selectedPlan,
    currentPlanTier,
    isExpired,
    error,
    preselectedId,
    showAllPlans,
    getActionType,
    onPlanSelect,
    onBackToSelected,
    className
}: PlanGridViewProps) {
    return (
        <div className={cn('space-y-8', className)}>
            <div className="text-center">
                {showAllPlans && preselectedId && (
                    <button
                        onClick={onBackToSelected}
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

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {plans.map((plan) => (
                    <PricingCard
                        key={plan.id}
                        card={plan}
                        isDisabled={false}
                        isSelected={selectedPlan?.id === plan.id}
                        onSelect={onPlanSelect}
                        actionType={getActionType(plan)}
                    />
                ))}
            </div>

            {error && (
                <div className="max-w-lg mx-auto bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 text-center">
                    <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
                </div>
            )}
        </div>
    );
}
