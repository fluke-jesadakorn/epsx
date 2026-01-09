
import { env } from '@/shared/env/schema';
import { PricingCardData } from '@/shared/types/plans';
import { DynamicPricingClient } from './DynamicPricingClient';

interface DynamicPricingSectionProps {
  initialAffiliateCode?: string | null;
}

export default async function DynamicPricingSection({ initialAffiliateCode }: DynamicPricingSectionProps = {}) {
  // Optimization: Do NOT read cookies here. Reading cookies forces dynamic rendering.
  // We strictly use the prop passed from the page (URL param).
  // If a user has a cookie but no URL param, the Client Component will handle fetching the discounted price.
  const affiliateCode = initialAffiliateCode || null;

  let personalPlans: PricingCardData[] = [];
  let apiPlans: PricingCardData[] = [];
  // const affiliateInfo = null; // To fully support affiliateInfo, we might need another fetch

  try {
    const baseUrl = env.BACKEND_URL || process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
    let apiUrl = `${baseUrl}/api/public/plans`;

    if (affiliateCode) {
      apiUrl += `?affiliate_code=${encodeURIComponent(affiliateCode)}`;
    }

    const response = await fetch(apiUrl, {
      method: 'GET',
      headers: { 'Accept': 'application/json' },
      // cache: 'force-cache' // or similar if we want caching
    });

    if (response.ok) {
      const result = await response.json();

      if (result.success && result.data && Array.isArray(result.data)) {
        const planData = result.data.map((item: any) => ({
          id: item.id,
          name: item.name,
          planType: item.plan_type,
          basePrice: parseFloat(item.current_price) || 0,
          currentPrice: parseFloat(item.current_price) || 0,
          effectivePrice: parseFloat(item.current_price) || 0,
          currency: item.currency || 'USD',
          displayOrder: item.display_order || 0,
          isActive: item.is_active,
          isHighlighted: item.is_highlighted || item.is_promoted || false,
          features: Array.isArray(item.features) ? item.features : [],
          activePromotions: [],
          promotionalBadges: [],
        }));

        // Use helper to transform (inline here since we can't easily share the one from Client)
        const transformToPricingCard = (plan: any): PricingCardData => {
          const hasDiscount = plan.currentPrice < plan.basePrice;

          return {
            id: plan.id,
            title: plan.name,
            price: plan.effectivePrice === 0 ? 'Free' : `$${plan.effectivePrice.toFixed(2)} ${plan.currency}`,
            originalPrice: hasDiscount ? `$${plan.basePrice.toFixed(2)} ${plan.currency}` : undefined,
            features: plan.features.map((feature: string) => ({ text: feature, included: true })),
            highlight: plan.isHighlighted,
            buttonText: plan.effectivePrice === 0 ? 'Start Free' : 'Get Started',
            promotions: plan.activePromotions || [],
            badges: plan.promotionalBadges || [],
            savings: hasDiscount ? `Save ${plan.currency} ${(plan.basePrice - plan.currentPrice).toFixed(2)}` : undefined
          };
        };

        personalPlans = planData
          .filter((plan: any) => plan.isActive)
          .filter((plan: any) => {
            const type = plan.planType?.toLowerCase();
            return !type || !type.includes('api');
          })
          .sort((a: any, b: any) => (a.displayOrder || 0) - (b.displayOrder || 0))
          .map(transformToPricingCard);

        apiPlans = planData
          .filter((plan: any) => plan.isActive)
          .filter((plan: any) => {
            const type = plan.planType?.toLowerCase();
            return type && type.includes('api');
          })
          .sort((a: any, b: any) => (a.displayOrder || 0) - (b.displayOrder || 0))
          .map(transformToPricingCard);
      }
    }
  } catch (error) {
    console.error('[DynamicPricing] Error fetching plans on server:', error);
  }

  return (
    <DynamicPricingClient
      personalPlans={personalPlans}
      apiPlans={apiPlans}
      affiliateCode={affiliateCode}
      affiliateInfo={null} // Pass null for now unless we fetch it 
    />
  );
}