'use client'

import { Check, Sparkles } from 'lucide-react'

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
}

interface PricingCardProps {
    card: PricingCardData
    onSelect: (card: PricingCardData) => void
    affiliateInfo?: any
    affiliateCode?: string | null
}

export function PricingCard({ card, onSelect, affiliateInfo, affiliateCode }: PricingCardProps) {
    return (
        <div className="relative">
            {/* Main Card */}
            <div
                className={`card-insight group relative overflow-visible h-full flex flex-col ${card.highlight
                    ? 'insight-gradient-soft-highlight ring-2 ring-orange-200/60 border-orange-200/50 dark:border-orange-400/30 shadow-2xl shadow-orange-500/25'
                    : 'ring-2 ring-blue-200/60 border-blue-200/50 dark:border-blue-400/30 shadow-xl shadow-blue-500/20'
                    }`}
            >
                {/* Card Content */}
                <div className="relative px-6 sm:px-8 pt-6 sm:pt-8 pb-6 sm:pb-8 flex flex-col h-full">
                    {/* Title Section */}
                    <div className="mb-4 h-[160px] flex flex-col items-center text-center">
                        <div className={`${card.highlight ? 'h-[80px]' : 'h-[40px]'} flex flex-col justify-start items-center mb-2`}>
                            <h3 className={`text-xl sm:text-2xl font-bold leading-tight whitespace-nowrap ${card.highlight
                                ? 'bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent'
                                : 'text-foreground'
                                } uppercase`}>
                                {card.title}
                            </h3>
                            {card.highlight && (
                                <div className="mt-2">
                                    <div className="bg-gradient-to-r from-orange-500 to-yellow-500 text-white px-3 py-1 rounded-full text-xs font-bold tracking-wide border-2 border-orange-300/50 shadow-lg shadow-orange-500/30">
                                        ⭐ MOST POPULAR ⭐
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* Price Display */}
                        <div className={`${card.highlight ? 'h-[58px]' : 'h-[78px]'} flex flex-col justify-center items-center`}>
                            <div className="flex items-baseline gap-3 flex-wrap justify-center">
                                <span className={`text-4xl sm:text-5xl font-bold leading-none whitespace-nowrap ${card.highlight
                                    ? 'bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent'
                                    : 'insight-gradient-text'
                                    }`}>
                                    {card.price}
                                </span>
                                {card.originalPrice && (
                                    <span className="text-lg text-gray-400 line-through decoration-2 whitespace-nowrap">
                                        {card.originalPrice}
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Features List */}
                    <div className="space-y-4 mb-8 flex-grow min-h-[200px] flex flex-col">
                        {card.features.map((feature, idx) => (
                            <div key={idx} className="flex items-start group/feature">
                                <div className={`flex-shrink-0 p-1.5 rounded-full ${card.highlight
                                    ? 'bg-orange-100 dark:bg-orange-900/30'
                                    : 'bg-insight-primary/20'
                                    }`}>
                                    <Check className={`h-4 w-4 ${card.highlight ? 'text-orange-600 dark:text-orange-400' : 'text-insight-primary'
                                        }`} />
                                </div>
                                <span className="ml-3 text-sm sm:text-base text-muted-foreground font-medium">
                                    {feature.text}
                                </span>
                            </div>
                        ))}
                    </div>

                    {/* Affiliate Info */}
                    {affiliateInfo && affiliateCode && (
                        <div className="mb-6 p-4 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-xl border border-green-200 dark:border-green-700">
                            <div className="flex items-center gap-2 text-sm text-green-700 dark:text-green-400">
                                <Sparkles className="h-4 w-4" />
                                <span className="font-semibold">Affiliate Bonus:</span>
                                <span>{affiliateInfo.commission_rate}% commission applied</span>
                            </div>
                        </div>
                    )}

                    {/* Action Button */}
                    <div className="mt-auto">
                        <button
                            className={`relative w-full rounded-xl font-semibold text-base py-4 overflow-hidden group ${card.highlight
                                ? 'bg-gradient-to-r from-orange-400 via-amber-400 via-yellow-400 via-amber-500 to-orange-500 hover:from-orange-500 hover:via-amber-500 hover:via-yellow-500 hover:via-amber-600 hover:to-orange-600 text-white shadow-xl shadow-orange-500/40 border-0'
                                : 'bg-gradient-to-r from-blue-400 via-cyan-400 via-blue-300 via-cyan-400 to-blue-400 hover:from-blue-500 hover:via-cyan-500 hover:via-blue-400 hover:via-cyan-500 hover:to-blue-500 text-white shadow-lg shadow-blue-400/30 border-0'
                                } before:absolute before:inset-0 before:bg-gradient-to-r before:from-transparent before:via-white/10 before:to-transparent before:translate-x-[-100%] hover:before:translate-x-[100%] before:transition-transform before:duration-700`}
                            onClick={() => onSelect(card)}
                        >
                            <span className="relative flex items-center justify-center gap-2">
                                {card.buttonText}
                                {card.highlight && <Sparkles className="h-4 w-4" />}
                            </span>
                        </button>
                    </div>
                </div>

                {/* Decorative Elements */}
                <div className="absolute -bottom-2 -right-2 w-12 h-12 bg-gradient-to-br from-transparent via-transparent to-gray-100/30 dark:to-gray-800/30 rounded-full blur-xl" />
                <div className="absolute -top-2 -left-2 w-8 h-8 bg-gradient-to-br from-transparent via-transparent to-blue-100/20 dark:to-blue-800/20 rounded-full blur-lg" />
            </div>
        </div>
    )
}

// Loading skeleton for pricing cards
export function PricingCardSkeleton() {
    return (
        <div className="card-insight">
            <div className="h-6 bg-gray-300 dark:bg-gray-700 rounded mb-4 animate-pulse" />
            <div className="h-8 bg-gray-300 dark:bg-gray-700 rounded mb-4 animate-pulse" />
            <div className="space-y-2 mb-6">
                <div className="h-4 bg-gray-300 dark:bg-gray-700 rounded animate-pulse" />
                <div className="h-4 bg-gray-300 dark:bg-gray-700 rounded animate-pulse" />
                <div className="h-4 bg-gray-300 dark:bg-gray-700 rounded animate-pulse" />
            </div>
            <div className="h-12 bg-gray-300 dark:bg-gray-700 rounded animate-pulse" />
        </div>
    )
}
