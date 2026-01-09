'use client'

import { cn } from '@/lib/utils'
import { Check, Clock, Crown, Lock, Sparkles, TrendingUp, Zap } from 'lucide-react'

export interface PricingCardData {
    id: number | string
    title: string
    price: string
    originalPrice?: string
    features: { text: string; included: boolean }[]
    highlight?: boolean
    buttonText: string
    promotions?: string[]
    badges?: string[]
    savings?: string
    tier_level?: number
    plan_type?: string
    description?: string
}

interface PricingCardProps {
    card: PricingCardData
    onSelect: (card: PricingCardData) => void
    affiliateInfo?: any
    affiliateCode?: string | null
    // Payment flow props
    isDisabled?: boolean
    isSelected?: boolean
    actionType?: 'extend' | 'upgrade' | 'downgrade' | 'locked' | 'select'
    className?: string
}

function getPlanIcon(planType: string = '') {
    const type = planType.toLowerCase();
    if (type.includes('enterprise') || type.includes('whale')) return <Crown className="w-6 h-6" />;
    if (type.includes('pro')) return <Zap className="w-6 h-6" />;
    return <Sparkles className="w-6 h-6" />;
}

export function PricingCard({
    card,
    onSelect,
    affiliateInfo,
    affiliateCode,
    isDisabled = false,
    isSelected = false,
    actionType = 'select',
    className
}: PricingCardProps) {

    // Styling based on state
    const getCardStyle = () => {
        if (isSelected) {
            return {
                borderClass: 'border-blue-500/20 ring-1 ring-blue-500/20 shadow-2xl shadow-blue-500/10',
                bgClass: 'bg-gradient-to-b from-blue-900/20 to-gray-900/80',
            }
        }
        if (isDisabled) {
            return {
                borderClass: 'border-white/5',
                bgClass: 'bg-gray-900/40',
            }
        }
        if (card.highlight) {
            return {
                borderClass: 'border-blue-500/30 shadow-2xl shadow-blue-900/20',
                bgClass: 'bg-gradient-to-b from-gray-800/90 to-gray-900/90',
            }
        }
        return {
            borderClass: 'border-white/5 hover:border-white/10',
            bgClass: 'bg-gray-900/60',
        }
    }

    const cardStyle = getCardStyle()

    return (
        <div
            className={cn(
                'relative rounded-2xl border transition-all duration-300 flex flex-col h-full',
                cardStyle.borderClass,
                cardStyle.bgClass,
                isDisabled ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer hover:shadow-xl hover:shadow-blue-500/10',
                className
            )}
            onClick={() => !isDisabled && onSelect(card)}
        >
            {/* Blur overlay for disabled cards */}
            {isDisabled && (
                <div className="absolute inset-0 z-10 backdrop-blur-[1px] bg-black/40 flex items-center justify-center rounded-2xl">
                    <div className="bg-gray-800 text-white px-4 py-2 rounded-lg flex items-center gap-2 shadow-xl border border-gray-700">
                        <Lock className="w-4 h-4" />
                        <span className="text-sm font-medium">Upgrade Only</span>
                    </div>
                </div>
            )}

            {/* Popular/Highlight badge */}
            {card.highlight && !isDisabled && (
                <div className="absolute -top-3 left-1/2 transform -translate-x-1/2 z-20">
                    <div className="bg-gradient-to-r from-blue-600 to-cyan-500 text-white text-xs font-bold px-4 py-1 rounded-full shadow-lg flex items-center gap-1 uppercase tracking-wider">
                        {card.title} PLAN ONLY
                    </div>
                </div>
            )}

            <div className="relative p-6 sm:p-8 flex flex-col h-full">
                {/* Header */}
                <div className="text-center mb-6">
                    <h3 className="text-lg font-bold text-gray-100 uppercase tracking-widest mb-4">
                        {card.title} PLAN
                    </h3>

                    <div className="flex items-center justify-center gap-2 mb-2">
                        <span className={cn(
                            "text-5xl sm:text-6xl font-black tracking-tighter",
                            card.highlight ? "text-blue-500" : "text-blue-500" // Consistent blue for price
                        )}>
                            {card.price === 'Free' ? 'Free' : card.price.split(' ')[0]}
                        </span>
                        {card.price !== 'Free' && (
                            <span className="text-xl font-bold text-blue-500 self-end mb-2">
                                USD
                            </span>
                        )}
                    </div>
                    {card.originalPrice && (
                        <p className="text-gray-500 line-through text-sm">
                            {card.originalPrice}
                        </p>
                    )}
                </div>

                {/* Features */}
                <div className="space-y-4 mb-8 flex-grow">
                    {card.features.map((feature, idx) => (
                        <div key={idx} className="flex items-start group/feature">
                            <div className="flex-shrink-0 mt-1">
                                <Check className="h-4 w-4 text-white" />
                            </div>
                            <span className="ml-3 text-sm text-gray-300 font-medium leading-normal">
                                {feature.text}
                            </span>
                        </div>
                    ))}
                </div>

                {/* Affiliate Info */}
                {affiliateInfo && affiliateCode && (
                    <div className="mb-6 p-4 bg-green-900/20 rounded-xl border border-green-800">
                        <div className="flex items-center gap-2 text-sm text-green-400">
                            <Sparkles className="h-4 w-4" />
                            <span className="font-semibold">Affiliate: {affiliateInfo.commission_rate}% applied</span>
                        </div>
                    </div>
                )}

                {/* Action button */}
                <div className="mt-auto">
                    <button
                        disabled={isDisabled}
                        className={cn(
                            'w-full py-4 rounded-xl font-bold text-base transition-all duration-300 relative overflow-hidden',
                            isDisabled
                                ? 'bg-gray-800 text-gray-500 cursor-not-allowed'
                                : 'bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:shadow-lg hover:shadow-cyan-500/25'
                        )}
                        onClick={(e) => {
                            e.stopPropagation(); // Prevent card click if button clicked
                            if (!isDisabled) onSelect(card);
                        }}
                    >
                        <span className="relative flex items-center justify-center gap-2">
                            {actionType === 'extend' && <Clock className="w-4 h-4" />}
                            {actionType === 'upgrade' && <TrendingUp className="w-4 h-4" />}
                            {actionType === 'downgrade' && <Sparkles className="w-4 h-4" />}
                            {actionType === 'locked' && <Lock className="w-4 h-4" />}

                            {card.buttonText}
                        </span>
                    </button>

                    {/* Helper text */}
                    {!isDisabled && actionType !== 'select' && (
                        <p className="text-center text-xs text-gray-500 mt-3">
                            {actionType === 'extend' && 'Extends current plan'}
                            {actionType === 'upgrade' && 'Pro-rated upgrade'}
                            {actionType === 'downgrade' && 'Switch plan'}
                        </p>
                    )}
                </div>
            </div>

            {/* Glow effect for highlighed cards */}
            {card.highlight && (
                <div className="absolute -inset-px bg-gradient-to-b from-blue-500/20 to-transparent rounded-2xl pointer-events-none -z-10" />
            )}
        </div>
    )
}
