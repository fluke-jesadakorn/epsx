'use client';

/**
 * PaymentPlanCard Component
 * 
 * Premium plan card for the payment page with upgrade/downgrade logic:
 * - Shows detailed plan info with features
 * - Disables (blurs) lower-tier plans during active subscription
 * - Allows extension if same plan selected
 * - Allows downgrade only when expired
 */

import { cn } from '@/lib/utils';
import { Check, Clock, Crown, Lock, Sparkles, TrendingUp, Zap } from 'lucide-react';

export interface PlanData {
    id: string | number;
    name: string;
    description?: string;
    plan_type: string;
    current_price: number;
    base_price?: number;
    currency: string;
    features: string[];
    is_highlighted?: boolean;
    is_promoted?: boolean;
    display_order?: number;
    tier_level?: number; // 1 = lowest, higher = better
}

interface PaymentPlanCardProps {
    plan: PlanData;
    currentPlanTier?: number;
    isExpired?: boolean;
    isSelected?: boolean;
    onSelect: (plan: PlanData) => void;
    className?: string;
}

/**
 * Get icon for plan type
 */
function getPlanIcon(planType: string) {
    const type = planType.toLowerCase();
    if (type.includes('enterprise') || type.includes('whale')) return <Crown className="w-6 h-6" />;
    if (type.includes('pro')) return <Zap className="w-6 h-6" />;
    return <Sparkles className="w-6 h-6" />;
}

export function PaymentPlanCard({
    plan,
    currentPlanTier = 0,
    isExpired = false,
    isSelected = false,
    onSelect,
    className,
}: PaymentPlanCardProps) {
    // Use tier_level from API, default to 0 (free tier) if not set
    const planTier = plan.tier_level ?? 0;

    // Determine card state
    const isCurrentPlan = planTier === currentPlanTier;
    const isDowngrade = planTier < currentPlanTier;
    const isUpgrade = planTier > currentPlanTier;

    // Card is disabled if attempting downgrade while plan is still active
    const isDisabled = isDowngrade && !isExpired;

    // Determine what action this represents
    const getActionType = () => {
        if (isCurrentPlan) return 'extend';
        if (isUpgrade) return 'upgrade';
        if (isDowngrade && isExpired) return 'downgrade';
        return 'locked';
    };

    const actionType = getActionType();

    // Has discount?
    const hasDiscount = plan.base_price && plan.base_price > plan.current_price;
    const discountPercent = hasDiscount
        ? Math.round((1 - plan.current_price / plan.base_price!) * 100)
        : 0;

    // Styling based on state
    const getCardStyle = () => {
        if (isSelected) {
            return {
                borderClass: 'border-purple-500 ring-2 ring-purple-500/30',
                bgClass: 'bg-gradient-to-br from-purple-50 via-white to-indigo-50 dark:from-purple-900/20 dark:via-gray-800 dark:to-indigo-900/20',
            };
        }
        if (isDisabled) {
            return {
                borderClass: 'border-gray-200 dark:border-gray-700',
                bgClass: 'bg-gray-100/80 dark:bg-gray-800/50',
            };
        }
        if (plan.is_highlighted) {
            return {
                borderClass: 'border-amber-300 dark:border-amber-600',
                bgClass: 'bg-gradient-to-br from-amber-50 via-white to-orange-50 dark:from-amber-900/20 dark:via-gray-800 dark:to-orange-900/20',
            };
        }
        return {
            borderClass: 'border-gray-200 dark:border-gray-700 hover:border-purple-300 dark:hover:border-purple-600',
            bgClass: 'bg-white/80 dark:bg-gray-800/80',
        };
    };

    const cardStyle = getCardStyle();

    return (
        <div
            className={cn(
                'relative rounded-2xl border-2 transition-all duration-300 overflow-hidden',
                cardStyle.borderClass,
                cardStyle.bgClass,
                isDisabled ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer hover:shadow-xl',
                className
            )}
            onClick={() => !isDisabled && onSelect(plan)}
        >
            {/* Blur overlay for disabled cards */}
            {isDisabled && (
                <div className="absolute inset-0 z-10 backdrop-blur-[2px] bg-white/20 dark:bg-black/20 flex items-center justify-center">
                    <div className="bg-gray-900/90 dark:bg-gray-800/95 text-white px-4 py-2 rounded-lg flex items-center gap-2 shadow-xl">
                        <Lock className="w-4 h-4" />
                        <span className="text-sm font-medium">Upgrade Only</span>
                    </div>
                </div>
            )}

            {/* Popular badge */}
            {plan.is_highlighted && !isDisabled && (
                <div className="absolute top-0 right-0 z-20">
                    <div className="bg-gradient-to-r from-amber-500 to-orange-500 text-white text-xs font-bold px-3 py-1 rounded-bl-xl shadow-lg flex items-center gap-1">
                        <Sparkles className="w-3 h-3" />
                        POPULAR
                    </div>
                </div>
            )}

            {/* Current plan badge */}
            {isCurrentPlan && !isDisabled && (
                <div className="absolute top-0 left-0 z-20">
                    <div className="bg-gradient-to-r from-emerald-500 to-teal-500 text-white text-xs font-bold px-3 py-1 rounded-br-xl shadow-lg flex items-center gap-1">
                        <Check className="w-3 h-3" />
                        CURRENT
                    </div>
                </div>
            )}

            <div className="relative p-6">
                {/* Header */}
                <div className="flex items-start justify-between mb-4">
                    <div className="flex items-center gap-3">
                        <div className={cn(
                            'flex h-12 w-12 items-center justify-center rounded-xl shadow-lg text-white',
                            plan.is_highlighted
                                ? 'bg-gradient-to-br from-amber-500 to-orange-600'
                                : 'bg-gradient-to-br from-purple-500 to-indigo-600'
                        )}>
                            {getPlanIcon(plan.plan_type)}
                        </div>
                        <div>
                            <h3 className={cn(
                                'text-xl font-black',
                                plan.is_highlighted
                                    ? 'bg-gradient-to-r from-amber-600 to-orange-600 bg-clip-text text-transparent'
                                    : 'text-gray-900 dark:text-white'
                            )}>
                                {plan.name}
                            </h3>
                            {plan.description && (
                                <p className="text-sm text-gray-500 dark:text-gray-400">
                                    {plan.description}
                                </p>
                            )}
                        </div>
                    </div>
                </div>

                {/* Price */}
                <div className="mb-6">
                    <div className="flex items-baseline gap-2">
                        <span className={cn(
                            'text-4xl font-black',
                            plan.is_highlighted
                                ? 'bg-gradient-to-r from-amber-500 to-orange-600 bg-clip-text text-transparent'
                                : 'text-gray-900 dark:text-white'
                        )}>
                            ${plan.current_price}
                        </span>
                        {hasDiscount && (
                            <span className="text-lg text-gray-400 line-through">
                                ${plan.base_price}
                            </span>
                        )}
                        <span className="text-gray-500 dark:text-gray-400 text-sm">
                            /{plan.currency || 'USD'}
                        </span>
                    </div>
                    {hasDiscount && (
                        <div className="inline-flex items-center gap-1 mt-2 px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 text-xs font-bold rounded-full">
                            <TrendingUp className="w-3 h-3" />
                            Save {discountPercent}%
                        </div>
                    )}
                </div>

                {/* Features */}
                <div className="space-y-3 mb-6">
                    {plan.features.slice(0, 5).map((feature, idx) => (
                        <div key={idx} className="flex items-start gap-3">
                            <div className={cn(
                                'flex-shrink-0 p-1 rounded-full',
                                plan.is_highlighted
                                    ? 'bg-amber-100 dark:bg-amber-900/30'
                                    : 'bg-purple-100 dark:bg-purple-900/30'
                            )}>
                                <Check className={cn(
                                    'w-3.5 h-3.5',
                                    plan.is_highlighted
                                        ? 'text-amber-600 dark:text-amber-400'
                                        : 'text-purple-600 dark:text-purple-400'
                                )} />
                            </div>
                            <span className="text-sm text-gray-700 dark:text-gray-300">
                                {feature}
                            </span>
                        </div>
                    ))}
                    {plan.features.length > 5 && (
                        <p className="text-xs text-purple-600 dark:text-purple-400 font-medium ml-8">
                            + {plan.features.length - 5} more features
                        </p>
                    )}
                </div>

                {/* Action button */}
                <button
                    disabled={isDisabled}
                    className={cn(
                        'w-full py-4 rounded-xl font-bold text-sm transition-all relative overflow-hidden',
                        isDisabled
                            ? 'bg-gray-300 dark:bg-gray-700 text-gray-500 dark:text-gray-400 cursor-not-allowed'
                            : isSelected
                                ? 'bg-gradient-to-r from-purple-600 to-indigo-600 text-white shadow-lg shadow-purple-500/30'
                                : plan.is_highlighted
                                    ? 'bg-gradient-to-r from-amber-500 to-orange-500 text-white shadow-lg shadow-amber-500/30 hover:shadow-xl'
                                    : 'bg-gradient-to-r from-purple-500 to-indigo-500 text-white shadow-lg hover:shadow-xl'
                    )}
                >
                    <span className="relative flex items-center justify-center gap-2">
                        {actionType === 'extend' && (
                            <>
                                <Clock className="w-4 h-4" />
                                Extend Plan
                            </>
                        )}
                        {actionType === 'upgrade' && (
                            <>
                                <TrendingUp className="w-4 h-4" />
                                Upgrade Now
                            </>
                        )}
                        {actionType === 'downgrade' && (
                            <>
                                <Sparkles className="w-4 h-4" />
                                Select Plan
                            </>
                        )}
                        {actionType === 'locked' && (
                            <>
                                <Lock className="w-4 h-4" />
                                Upgrade Required
                            </>
                        )}
                        {isSelected && (
                            <Check className="w-4 h-4 ml-1" />
                        )}
                    </span>
                </button>

                {/* Upgrade/extend hint */}
                {!isDisabled && (
                    <p className="text-center text-xs text-gray-500 dark:text-gray-400 mt-3">
                        {actionType === 'extend'
                            ? '💫 Extends your current plan by 30 days'
                            : actionType === 'upgrade'
                                ? '✨ Pro-rata credit applied from current plan'
                                : actionType === 'downgrade'
                                    ? '🔄 Switch to this plan'
                                    : ''}
                    </p>
                )}
            </div>
        </div>
    );
}

export default PaymentPlanCard;
