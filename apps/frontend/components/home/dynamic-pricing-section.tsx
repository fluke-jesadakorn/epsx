 
import { getPublicPlansAction } from '@/app/actions/plans';
import type { PricingCardData } from '@/shared/types/plans';
import { DynamicPricingClient } from './dynamic-pricing-client';

interface DynamicPricingSectionProps {
  initialAffiliateCode?: string | null;
}

interface PlanResponse {
  id: string;
  name: string;
  plan_type: string;
  plan_group?: string;
  current_price: string | number;
  effective_price?: number;
  promotion_active?: boolean;
  promotion_discount?: number;
  promotion_ends_at?: string;
  currency?: string;
  display_order?: number;
  is_active: boolean;
  is_highlighted?: boolean;
  is_promoted?: boolean;
  features?: string[];
}

export default async function DynamicPricingSection({ initialAffiliateCode }: DynamicPricingSectionProps = {}) {
  // Optimization: Do NOT read cookies here. Reading cookies forces dynamic rendering.
  // We strictly use the prop passed from the page (URL param).
  // If a user has a cookie but no URL param, the Client Component will handle fetching the discounted price.
  const affiliateCode = initialAffiliateCode ?? null;

  let personalPlans: PricingCardData[] = [];
  let enterprisePlans: PricingCardData[] = [];
  let apiPlans: PricingCardData[] = [];

  try {
    const response = await getPublicPlansAction({
      affiliate_code: affiliateCode ?? undefined
    });

    if (response.success && response.data && Array.isArray(response.data)) {
      const planData = response.data.map((item: PlanResponse) => {
        const basePrice = parseFloat(String(item.current_price)) || 0;
        const hasPromo = item.promotion_active === true &&
          item.effective_price !== undefined &&
          item.effective_price < basePrice;
        return {
          id: item.id,
          name: item.name,
          planType: item.plan_type,
          planGroup: item.plan_group ?? 'personal',
          basePrice,
          effectivePrice: hasPromo ? (item.effective_price ?? basePrice) : basePrice,
          currency: item.currency ?? 'USD',
          displayOrder: item.display_order ?? 0,
          isActive: item.is_active,
          isHighlighted: item.is_highlighted ?? item.is_promoted ?? false,
          features: Array.isArray(item.features) ? item.features : [],
          hasPromo,
          promoDiscount: item.promotion_discount ?? 0,
          promoEndsAt: item.promotion_ends_at,
        };
      });

      const transformToPricingCard = (plan: typeof planData[number]): PricingCardData => ({
        id: plan.id,
        title: plan.name,
        price: plan.effectivePrice === 0 ? 'Free' : `$${plan.effectivePrice.toFixed(2)} ${plan.currency}`,
        originalPrice: plan.hasPromo ? `$${plan.basePrice.toFixed(2)} ${plan.currency}` : undefined,
        features: plan.features.map((feature: string) => ({ text: feature, included: true })),
        highlight: plan.isHighlighted,
        buttonText: plan.effectivePrice === 0 ? 'Start Free' : 'Get Started',
        promotions: plan.hasPromo ? [`${Math.round(plan.promoDiscount)}% OFF`] : [],
        badges: [],
        savings: plan.hasPromo ? `Save $${(plan.basePrice - plan.effectivePrice).toFixed(2)}` : undefined,
        promotion_ends_at: plan.hasPromo ? plan.promoEndsAt : undefined,
      });

      const active = planData.filter((p) => p.isActive);
      const sorted = (items: typeof active) => items.sort((a, b) => (a.displayOrder ?? 0) - (b.displayOrder ?? 0));

      personalPlans = sorted(active.filter((p) => p.planGroup === 'personal')).map(transformToPricingCard);
      enterprisePlans = sorted(active.filter((p) => p.planGroup === 'enterprise')).map(transformToPricingCard);
      apiPlans = sorted(active.filter((p) => p.planGroup === 'api')).map(transformToPricingCard);
    }
  } catch (_error) {
      // Error logged silently
  }

  return (
    <DynamicPricingClient
      personalPlans={personalPlans}
      enterprisePlans={enterprisePlans}
      apiPlans={apiPlans}
      affiliateCode={affiliateCode}
      affiliateInfo={null}
    />
  );
}