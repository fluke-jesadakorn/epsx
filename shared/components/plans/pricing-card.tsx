'use client'

import { cn } from '@/shared/utils'
import { getTimeRemaining } from '@/shared/utils/promo'
import { Check, Clock, Flame, Lock, Sparkles, TrendingUp } from 'lucide-react'

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
    promotion_ends_at?: string
    tier_level?: number
    plan_type?: string
    description?: string
}

interface PricingCardProps {
    card: PricingCardData
    onSelect: (card: PricingCardData) => void
    affiliateInfo?: { commission_rate: number }
    affiliateCode?: string | null
    // Payment flow props
    isDisabled?: boolean
    isSelected?: boolean
    actionType?: 'extend' | 'upgrade' | 'downgrade' | 'locked' | 'select'
    className?: string
    // Credit balance
    creditBalance?: number
}

export function PricingCard({
    card,
    onSelect,
    affiliateInfo,
    affiliateCode,
    isDisabled = false,
    isSelected = false,
    actionType = 'select',
    className,
    creditBalance
}: PricingCardProps) {
    // Calculate credit application
    const planPrice = card.price === 'Free'
        ? 0
        : parseFloat(card.price.replace(/[^0-9.]/g, ''))

    const hasCredits = (creditBalance ?? 0) > 0
    const creditsCover = hasCredits && creditBalance !== undefined ? Math.min(creditBalance, planPrice) : 0
    const amountDue = Math.max(0, planPrice - creditsCover)

    // Styling based on state
    const getCardStyle = () => {
        if (isSelected) {
            return {
                borderClass: 'border-blue-500/20 ring-1 ring-blue-500/20 shadow-2xl shadow-blue-500/10',
                bgClass: 'bg-white dark:bg-slate-900 dark:border-blue-900/30',
            }
        }
        if (isDisabled) {
            return {
                borderClass: 'border-gray-200 dark:border-slate-700',
                bgClass: 'bg-gray-50 dark:bg-slate-900',
            }
        }
        if (card.highlight === true) {
            return {
                borderClass: 'border-blue-500/30 shadow-2xl shadow-blue-900/20',
                bgClass: 'bg-white dark:bg-slate-900',
            }
        }
        return {
            borderClass: 'border-gray-200 dark:border-slate-700 hover:border-gray-300 dark:hover:border-white/20',
            bgClass: 'bg-white dark:bg-slate-900',
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
                <div className="absolute inset-0 z-10 backdrop-blur-[1px] bg-white/40 dark:bg-black/40 flex items-center justify-center rounded-2xl">
                    <div className="bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white px-4 py-2 rounded-lg flex items-center gap-2 shadow-xl border border-gray-200 dark:border-gray-700">
                        <Lock className="w-4 h-4" />
                        <span className="text-sm font-medium">Upgrade Only</span>
                    </div>
                </div>
            )}

            {/* Sale ribbon */}
            {(card.promotions?.length ?? 0) > 0 && !isDisabled && (
                <div className="absolute -top-px -left-px z-20 bg-red-500 text-white text-xs font-bold px-3 py-1 rounded-br-lg rounded-tl-2xl uppercase tracking-wider">
                    SALE
                </div>
            )}

            {/* Popular/Highlight badge */}
            {card.highlight === true && !isDisabled && (
                <div className="absolute -top-3 left-1/2 transform -translate-x-1/2 z-20">
                    <div className="bg-gradient-to-r from-blue-600 to-cyan-500 text-white text-xs font-bold px-4 py-1 rounded-full shadow-lg flex items-center gap-1 uppercase tracking-wider">
                        {card.title} ONLY
                    </div>
                </div>
            )}

            <div className="relative p-6 sm:p-8 flex flex-col h-full">
                <PricingCardHeader card={card} />

                <PricingCardFeatures features={card.features} />

                {/* Affiliate Info */}
                {affiliateInfo !== undefined && affiliateCode !== undefined && affiliateCode !== null && (
                    <div className="mb-6 p-4 bg-green-50 dark:bg-green-900/20 rounded-xl border border-green-200 dark:border-green-800">
                        <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                            <Sparkles className="h-4 w-4" />
                            <span className="font-semibold">Affiliate: {affiliateInfo.commission_rate}% applied</span>
                        </div>
                    </div>
                )}

                {/* Credit Balance Info */}
                {hasCredits && planPrice > 0 && (
                    <div className="mb-6 p-4 bg-emerald-50 dark:bg-emerald-900/30 rounded-xl border border-emerald-200 dark:border-emerald-700">
                        <div className="flex items-center gap-2 mb-3">
                            <Sparkles className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
                            <span className="text-sm font-semibold text-emerald-600 dark:text-emerald-400">Credits Applied</span>
                        </div>
                        <div className="space-y-1.5 text-sm">
                            <div className="flex justify-between text-gray-600 dark:text-gray-300">
                                <span>Plan Price:</span>
                                <span className="font-mono">${planPrice.toFixed(2)}</span>
                            </div>
                            <div className="flex justify-between text-emerald-600 dark:text-emerald-400">
                                <span>Credit Applied:</span>
                                <span className="font-mono">-${creditsCover.toFixed(2)}</span>
                            </div>
                            <div className="h-px bg-emerald-200 dark:bg-emerald-700 my-2" />
                            <div className="flex justify-between font-bold text-gray-900 dark:text-white">
                                <span>Amount Due:</span>
                                <span className="font-mono text-lg">${amountDue.toFixed(2)}</span>
                            </div>
                            {amountDue === 0 && (
                                <div className="text-center mt-2 text-xs text-emerald-600 dark:text-emerald-400 font-semibold">
                                    ✨ Fully covered by credits!
                                </div>
                            )}
                        </div>
                    </div>
                )}

                <PricingCardButton
                    card={card}
                    isDisabled={isDisabled}
                    actionType={actionType}
                    onSelect={onSelect}
                />
            </div>

            {/* Glow effect for highlighted cards */}
            {card.highlight === true && (
                <div className="absolute -inset-px bg-gradient-to-b from-blue-500/20 to-transparent rounded-2xl pointer-events-none -z-10" />
            )}
        </div>
    )
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function PricingCardHeader({ card }: { card: PricingCardData }) {
    const promoBadge = card.promotions?.[0]
    const timeLeft = card.promotion_ends_at !== undefined ? getTimeRemaining(card.promotion_ends_at) : null

    return (
        <div className="text-center mb-6">
            <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100 uppercase tracking-widest mb-4">
                {card.title}
            </h3>

            <div className="flex items-center justify-center gap-2 mb-2 min-w-0">
                <span className="text-3xl sm:text-4xl font-black tracking-tighter text-blue-500 whitespace-nowrap">
                    {card.price === 'Free' ? 'Free' : card.price.split(' ')[0]}
                </span>
                {card.price !== 'Free' && (
                    <span className="text-base font-bold text-blue-500 self-end mb-1">
                        USD
                    </span>
                )}
                {promoBadge !== undefined && (
                    <span className="bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full self-center">
                        {promoBadge}
                    </span>
                )}
            </div>

            {card.originalPrice !== undefined && card.originalPrice !== '' && (
                <p className="text-gray-500 line-through text-sm">
                    {card.originalPrice}
                </p>
            )}

            {card.savings !== undefined && card.savings !== '' && (
                <p className="text-green-500 text-sm font-semibold mt-1">
                    {card.savings}
                </p>
            )}

            {timeLeft !== null && timeLeft !== 'Expired' && (
                <p className="text-orange-400 text-xs mt-1 flex items-center justify-center gap-1">
                    <Flame className="w-3 h-3" />
                    Ends in {timeLeft.replace(' left', '')}
                </p>
            )}
        </div>
    )
}

function PricingCardFeatures({ features }: { features: PricingCardData['features'] }) {
    return (
        <div className="space-y-4 mb-8 flex-grow">
            {features.map((feature) => (
                <div key={feature.text} className="flex items-start group/feature">
                    <div className="flex-shrink-0 mt-1">
                        <Check className="h-4 w-4 text-blue-600 dark:text-white" />
                    </div>
                    <span className="ml-3 text-sm text-gray-600 dark:text-gray-300 font-medium leading-normal">
                        {feature.text}
                    </span>
                </div>
            ))}
        </div>
    )
}

function PricingCardButton({
    isDisabled,
    actionType,
    card,
    onSelect
}: {
    isDisabled: boolean
    actionType: string
    card: PricingCardData
    onSelect: (card: PricingCardData) => void
}) {
    return (
        <div className="mt-auto">
            <button
                disabled={isDisabled}
                className={cn(
                    'w-full py-4 rounded-xl font-bold text-base transition-all duration-300 relative overflow-hidden',
                    isDisabled
                        ? 'bg-gray-200 dark:bg-gray-800 text-gray-400 dark:text-gray-500 cursor-not-allowed'
                        : 'bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:shadow-lg hover:shadow-cyan-500/25'
                )}
                onClick={(e) => {
                    e.stopPropagation(); // Prevent card click if button clicked
                    if (!isDisabled) { onSelect(card); }
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
    )
}
