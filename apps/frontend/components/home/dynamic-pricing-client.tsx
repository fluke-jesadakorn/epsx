'use client';

import { PricingCard } from '@/shared/components/plans/pricing-card';
import type { PricingCardData } from '@/shared/types/plans';
import { Star } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';

interface AffiliateInfo {
    commission_rate?: number;
}

interface ApiPlanData {
    id: string;
    name: string;
    plan_type: string;
    is_active: boolean;
    display_order: number;
    currentPrice: number;
    basePrice: number;
    effectivePrice: number;
    currency: string;
    features: string[];
    isHighlighted?: boolean;
    is_highlighted?: boolean;
    is_promoted?: boolean;
    activePromotions?: unknown[];
    promotionalBadges?: unknown[];
}

interface ApiResponse {
    success: boolean;
    data: ApiPlanData[];
}

interface DynamicPricingClientProps {
    personalPlans: PricingCardData[];
    apiPlans: PricingCardData[];
    affiliateCode: string | null;
    affiliateInfo: AffiliateInfo | null;
}

const transformToPricingCard = (plan: ApiPlanData): PricingCardData => {
    const hasDiscount = plan.currentPrice < plan.basePrice;
    return {
        id: plan.id,
        title: plan.name,
        price: plan.effectivePrice === 0 ? 'Free' : `$${plan.effectivePrice.toFixed(2)} ${plan.currency}`,
        originalPrice: hasDiscount ? `$${plan.basePrice.toFixed(2)} ${plan.currency}` : undefined,
        features: plan.features.map((feature: string) => ({ text: feature, included: true })),
        highlight: plan.isHighlighted ?? plan.is_highlighted ?? plan.is_promoted ?? false,
        buttonText: plan.effectivePrice === 0 ? 'Start Free' : 'Get Started',
        promotions: plan.activePromotions ?? [],
        badges: plan.promotionalBadges ?? [],
        savings: hasDiscount ? `Save ${plan.currency} ${(plan.basePrice - plan.currentPrice).toFixed(2)}` : undefined
    };
};

const filterPersonalPlans = (plans: ApiPlanData[]): ApiPlanData[] => {
    return plans.filter((plan) => {
        if (!plan.is_active) {
            return false;
        }
        const type = plan.plan_type.toLowerCase();
        return !type.includes('api');
    });
};

const filterApiPlans = (plans: ApiPlanData[]): ApiPlanData[] => {
    return plans.filter((plan) => {
        if (!plan.is_active) {
            return false;
        }
        const type = plan.plan_type.toLowerCase();
        return type.includes('api');
    });
};

const sortByDisplayOrder = (plans: ApiPlanData[]): ApiPlanData[] => {
    return plans.sort((a, b) => a.display_order - b.display_order);
};

const BackgroundDecorations = (): JSX.Element => (
    <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 dark:from-orange-600/5 dark:to-yellow-600/5 rounded-full animate-float" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 dark:from-blue-700/5 dark:to-cyan-700/5 rounded-full animate-bounce-gentle" />
        <div className="absolute top-1/2 left-1/4 w-16 h-16 bg-gradient-to-br from-purple-400/10 to-pink-400/10 dark:from-purple-700/5 dark:to-pink-700/5 rounded-full animate-pulse-gentle" />
        <div className="absolute bottom-1/4 right-1/3 w-20 h-20 bg-gradient-to-br from-green-400/10 to-emerald-400/10 dark:from-green-700/5 dark:to-emerald-700/5 rounded-full animate-float-reverse" />
    </div>
);

interface AffiliateBannerProps {
    code: string;
    commissionRate: number;
}

const AffiliateBanner = ({ code, commissionRate }: AffiliateBannerProps): JSX.Element => (
    <div className="container mx-auto px-4 mb-8">
        <div className="bg-gradient-to-r from-purple-500 to-pink-500 text-white p-4 rounded-2xl text-center shadow-xl">
            <div className="flex items-center justify-center gap-2 text-lg font-semibold">
                <Star className="h-5 w-5" />
                <span>You&apos;re eligible for {commissionRate}% affiliate rewards!</span>
                <Star className="h-5 w-5" />
            </div>
            <p className="text-sm mt-1 opacity-90">
                Referred by partner: {code} • Special pricing applied
            </p>
        </div>
    </div>
);

const PersonalPlansHeader = (): JSX.Element => (
    <div className="text-center space-y-6 sm:space-y-8 animate-slide-up">
        <h2 className="text-4xl sm:text-6xl font-bold">
            <span className="mr-2">💰</span>
            <span className="bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
                Personal Plans
            </span>
        </h2>
        <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
            🚀 Choose the perfect plan for individual use and start your data journey
        </p>
        <div className="w-32 sm:w-40 h-1.5 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 mx-auto rounded-full" />
        <div className="flex justify-center items-center gap-4 mt-6">
            <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
            <div className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse" style={{ animationDelay: '0.5s' }} />
            <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" style={{ animationDelay: '1s' }} />
        </div>
    </div>
);

const ApiPlansHeader = (): JSX.Element => (
    <div className="text-center space-y-4 sm:space-y-6 animate-fade-in">
        <h2 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-orange-500 via-pink-500 to-purple-500 bg-clip-text text-transparent">
            API Plans
        </h2>
        <p className="text-lg sm:text-xl text-foreground/80 max-w-2xl mx-auto">
            Integrate our powerful API into your systems
        </p>
        <div className="relative w-24 sm:w-32 h-1 sm:h-1.5 bg-gradient-to-r from-orange-500 to-pink-500 mx-auto rounded-full overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/25 to-transparent animate-shimmer" />
        </div>
    </div>
);

export const DynamicPricingClient = ({
    personalPlans: initialPersonalPlans,
    apiPlans: initialApiPlans,
    affiliateCode: initialAffiliateCode,
    affiliateInfo: initialAffiliateInfo
}: DynamicPricingClientProps) => {
    const router = useRouter();
    const searchParams = useSearchParams();

    const [personalPlans, setPersonalPlans] = useState<PricingCardData[]>(initialPersonalPlans);
    const [apiPlans, setApiPlans] = useState<PricingCardData[]>(initialApiPlans);
    const [affiliateCode, setAffiliateCode] = useState<string | null>(initialAffiliateCode);
    const [_affiliateInfo, _setAffiliateInfo] = useState<AffiliateInfo | null>(initialAffiliateInfo);

    // Helper to fetch updated plans client-side
    const fetchPlansWithCode = async (code: string): Promise<void> => {
        try {
            const baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
            const response = await fetch(`${baseUrl}/api/public/plans?affiliate_code=${encodeURIComponent(code)}`);
            if (!response.ok) {
                return;
            }

            const result: unknown = await response.json();
            if (typeof result !== 'object' || result === null) {
                return;
            }

            const typedResult = result as ApiResponse;
            if (!typedResult.success || !Array.isArray(typedResult.data)) {
                return;
            }

            const planData = typedResult.data;
            const newPersonalPlans = sortByDisplayOrder(filterPersonalPlans(planData)).map(transformToPricingCard);
            const newApiPlans = sortByDisplayOrder(filterApiPlans(planData)).map(transformToPricingCard);

            setPersonalPlans(newPersonalPlans);
            setApiPlans(newApiPlans);
        } catch (_err) {
            // Error logged silently
        }
    };

    // Extract affiliate code from URL parameters and set cookie
    useEffect(() => {
        const refCode = searchParams.get('ref') ?? searchParams.get('affiliate') ?? searchParams.get('aff');
        if (refCode !== null && refCode !== '') {
            document.cookie = `affiliate_code=${encodeURIComponent(refCode)}; path=/; max-age=2592000; SameSite=lax`;
            // Also update state to reflect immediate change if we weren't already using it
            if (refCode !== affiliateCode) {
                setAffiliateCode(refCode);
                void fetchPlansWithCode(refCode);
            }
        } else if (affiliateCode === null || affiliateCode === '') {
            // No URL code, but maybe we have a cookie that the server ignored (for static optimization)
            const match = document.cookie.match(new RegExp('(^| )affiliate_code=([^;]+)'));
            if (match !== null) {
                const cookieCode = decodeURIComponent(match[2]);
                setAffiliateCode(cookieCode);
                void fetchPlansWithCode(cookieCode);
            }
        }
    }, [searchParams, affiliateCode]);

    const handlePlanClick = (plan: PricingCardData): void => {
        // REFACTOR: Use dynamic routes instead of query strings
        let paymentUrl = `/payment/plan/${plan.id}`;

        // We still support affiliate codes via query string on the dynamic route
        if (affiliateCode !== null && affiliateCode !== '') {
            paymentUrl += `?ref=${encodeURIComponent(affiliateCode)}`;
        }

        router.push(paymentUrl);
    };

    const renderPricingCards = (cards: PricingCardData[]): JSX.Element[] => {
        return cards.map((card) => (
            <PricingCard
                key={card.id}
                card={card}
                onSelect={handlePlanClick}
                affiliateInfo={_affiliateInfo}
                affiliateCode={affiliateCode}
            />
        ));
    };

    const showAffiliateBanner = affiliateCode !== null && affiliateCode !== '' && _affiliateInfo !== null;

    return (
        <div className="relative w-full py-16 sm:py-24 lg:py-32 overflow-hidden">
            <BackgroundDecorations />

            {showAffiliateBanner && (
                <AffiliateBanner
                    code={affiliateCode}
                    commissionRate={_affiliateInfo.commission_rate ?? 0}
                />
            )}

            <div className="relative container mx-auto px-4 sm:px-6 lg:px-8 max-w-7xl space-y-16 sm:space-y-20 lg:space-y-24">
                <div className="space-y-8 sm:space-y-12">
                    <PersonalPlansHeader />
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-12 lg:gap-16 px-4 py-8">
                        {renderPricingCards(personalPlans)}
                    </div>
                </div>

                <div className="space-y-8 sm:space-y-12">
                    <ApiPlansHeader />
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-12 lg:gap-16 px-4 py-8">
                        {renderPricingCards(apiPlans)}
                    </div>
                </div>
            </div>
        </div>
    );
};
