import { Zap } from 'lucide-react';
import type { PaymentPackage, ApiPaymentPlan } from './types';

export const getIconForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return '🚀';
    case 'professional':
    case 'pro':
      return '⭐';
    case 'enterprise':
    case 'premium':
      return '👑';
    default:
      return '📊';
  }
};

export const getDescriptionForPlan = (planType: string): string => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return 'Perfect for beginners';
    case 'professional':
    case 'pro':
      return 'Most popular choice';
    case 'enterprise':
    case 'premium':
      return 'For serious analysts';
    default:
      return 'Analytics plan';
  }
};

export const getDefaultFeaturesForPlan = (planType: string): string[] => {
  switch (planType.toLowerCase()) {
    case 'starter':
    case 'basic':
      return [
        '5 API calls per day',
        'Basic analytics dashboard',
        'Email support',
        'Mobile app access',
        'Basic stock alerts',
      ];
    case 'professional':
    case 'pro':
      return [
        'Everything in Starter',
        '50 API calls per day',
        'Advanced analytics & charts',
        'Priority email support',
        'Real-time data streaming',
        'Portfolio tracking',
        'Custom alerts & notifications',
      ];
    case 'enterprise':
    case 'premium':
      return [
        'Everything in Professional',
        'Unlimited API calls',
        'Premium analytics suite',
        '24/7 phone & chat support',
        'AI-powered insights',
        'Advanced portfolio management',
        'Custom integrations',
        'Dedicated account manager',
      ];
    default:
      return ['Standard features included'];
  }
};

export const PAYMENT_METHODS = [
  {
    id: 'metamask',
    name: 'MetaMask (Instant)',
    icon: Zap,
    description: 'Pay directly with USDT/USDC via MetaMask',
  },
] as const;

const parsePrice = (
  currentPrice: number | string,
  effectivePrice?: number
): number => {
  let price: number;

  if (typeof currentPrice === 'string') {
    price = parseFloat(currentPrice);
  } else if (typeof currentPrice === 'number') {
    price = currentPrice;
  } else {
    price = 0;
  }

  if ((isNaN(price) || price < 0) && typeof effectivePrice === 'number' && effectivePrice >= 0) {
    price = effectivePrice;
  }

  return (isNaN(price) || price < 0) ? 0 : price;
};

// API helper function to fetch and transform plans
export const fetchPlans = async (): Promise<PaymentPackage[]> => {
  const { getPublicPlansAction } = await import('@/app/actions/plans');
  const result = await getPublicPlansAction();

  if (!result.success || !result.data || !Array.isArray(result.data)) {
    throw new Error(
      typeof result.message === 'string' ? result.message : 'Invalid API response format'
    );
  }

  const plans = result.data as ApiPaymentPlan[];

  return plans.map((plan: ApiPaymentPlan, _index: number): PaymentPackage => {
      const currentPrice = parsePrice(plan.current_price, plan.effective_price);

      const basePrice = plan.base_price ?? currentPrice;

      const effectivePrice = plan.effective_price ?? currentPrice;
      const originalPlanId = plan.id;

      const features = Array.isArray(plan.features)
        ? plan.features
        : typeof plan.features === 'string'
          ? JSON.parse(plan.features) as string[]
          : getDefaultFeaturesForPlan(plan.plan_type);

      return {
        ...plan,
        id: originalPlanId,
        original_plan_id: originalPlanId,
        current_price: currentPrice,
        base_price: basePrice,
        effective_price: effectivePrice,
        icon: getIconForPlan(plan.plan_type),
        description: getDescriptionForPlan(plan.plan_type),
        popular: (plan.is_highlighted ?? false) || plan.plan_type === 'professional',
        features,
      };
    });
};
