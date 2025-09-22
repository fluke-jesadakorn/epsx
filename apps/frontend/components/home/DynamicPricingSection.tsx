'use client';

import { Check, Sparkles, Tag, Star } from 'lucide-react';
import React, { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { env } from '../../../../shared/env/schema';

interface PlanFeature {
  text: string;
  included: boolean;
}

interface DynamicPlan {
  id: number;
  name: string;
  planType: 'personal' | 'api';
  basePrice: number;
  currentPrice: number;
  currency: string;
  features: string[];
  affiliateCommissionRate: number;
  displayOrder: number;
  isActive: boolean;
  isHighlighted: boolean;
  activePromotions: string[];
  effectivePrice: number;
  promotionalBadges?: string[];
  campaignSummary?: string;
}

interface DynamicPricingCard {
  id: number;
  title: string;
  price: string;
  originalPrice?: string;
  features: PlanFeature[];
  highlight?: boolean;
  buttonText: string;
  buttonVariant?: 'default' | 'outline';
  promotions: string[];
  badges: string[];
  savings?: string;
}

const DynamicPricingSection = () => {
  const [personalPlans, setPersonalPlans] = useState<DynamicPricingCard[]>([]);
  const [apiPlans, setApiPlans] = useState<DynamicPricingCard[]>([]);
  const [loading, setLoading] = useState(true);
  const [affiliateCode, setAffiliateCode] = useState<string | null>(null);
  const [affiliateInfo, setAffiliateInfo] = useState<any>(null);
  
  const router = useRouter();
  const searchParams = useSearchParams();

  // Extract affiliate code from URL parameters
  useEffect(() => {
    const refCode = searchParams.get('ref') || searchParams.get('affiliate') || searchParams.get('aff');
    if (refCode) {
      setAffiliateCode(refCode);
      // Store in localStorage for persistence
      localStorage.setItem('affiliateCode', refCode);
    } else {
      // Check localStorage for existing affiliate code
      const storedCode = localStorage.getItem('affiliateCode');
      if (storedCode) {
        setAffiliateCode(storedCode);
      }
    }
  }, [searchParams]);

  // Fetch dynamic plans from backend
  useEffect(() => {
    const fetchPlans = async () => {
      try {
        setLoading(true);
        
        // Build API URL with affiliate tracking
        const baseUrl = env.BACKEND_URL;
        let apiUrl = `${baseUrl}/api/v1/plans`;
        
        // Add affiliate code if available
        if (affiliateCode) {
          apiUrl += `?affiliate_code=${encodeURIComponent(affiliateCode)}`;
        }

        const response = await fetch(apiUrl);
        
        if (!response.ok) {
          throw new Error('Failed to fetch plans');
        }
        
        const result = await response.json();
        
        if (result.success && result.data) {
          const planData = result.data
            .map((item: any) => ({
              id: item.id,
              name: item.name,
              planType: item.plan_type,
              basePrice: parseFloat(item.current_price) || 0,
              currentPrice: parseFloat(item.current_price) || 0,
              effectivePrice: parseFloat(item.current_price) || 0,
              currency: item.currency || 'USD',
              displayOrder: item.display_order || 0,
              isActive: item.is_active,
              isHighlighted: item.is_highlighted || false,
              features: Array.isArray(item.features) ? item.features : [],
              activePromotions: [],
              promotionalBadges: [],
              campaignSummary: item.description,
              affiliateCommissionRate: 0
            }));
          
          // Separate personal and API plans
          const personal = planData
            .filter(plan => plan.planType === 'personal' && plan.isActive)
            .sort((a, b) => (a.displayOrder || 0) - (b.displayOrder || 0))
            .map(transformToPricingCard);
            
          const api = planData
            .filter(plan => plan.planType === 'api' && plan.isActive)
            .sort((a, b) => (a.displayOrder || 0) - (b.displayOrder || 0))
            .map(transformToPricingCard);
          
          setPersonalPlans(personal);
          setApiPlans(api);

          // If affiliate code provided, try to fetch affiliate info
          if (affiliateCode && planData.length > 0) {
            fetchAffiliateInfo(affiliateCode);
          }
        } else {
          // If no valid data from API, use fallback
          throw new Error('No valid plan data received');
        }
      } catch (error) {
        console.error('Error fetching dynamic plans:', error);
        // Fallback to static plans if API fails
        loadFallbackPlans();
      } finally {
        setLoading(false);
      }
    };

    fetchPlans();
  }, [affiliateCode]);

  // Fetch affiliate information for commission display
  const fetchAffiliateInfo = async (code: string) => {
    try {
      const baseUrl = env.BACKEND_URL;
      const response = await fetch(`${baseUrl}/api/v1/plans/calculate-price/1?affiliate_code=${code}`);
      
      if (response.ok) {
        const result = await response.json();
        if (result.success && result.data?.affiliate) {
          setAffiliateInfo(result.data.affiliate);
        }
      }
    } catch (error) {
      console.error('Error fetching affiliate info:', error);
    }
  };

  // Transform backend plan to frontend pricing card
  const transformToPricingCard = (plan: DynamicPlan): DynamicPricingCard => {
    const hasDiscount = plan.currentPrice < plan.basePrice;
    const savings = hasDiscount 
      ? `Save ${plan.currency} ${(plan.basePrice - plan.currentPrice).toFixed(2)}`
      : undefined;

    return {
      id: plan.id,
      title: plan.name,
      price: `$${plan.effectivePrice.toFixed(2)} ${plan.currency}`,
      originalPrice: hasDiscount ? `$${plan.basePrice.toFixed(2)} ${plan.currency}` : undefined,
      features: plan.features.map(feature => ({ text: feature, included: true })),
      highlight: plan.isHighlighted,
      buttonText: plan.effectivePrice === 0 ? 'Start Free' : 'Get Started',
      buttonVariant: plan.effectivePrice === 0 ? 'outline' : 'default',
      promotions: plan.activePromotions || [],
      badges: plan.promotionalBadges || [],
      savings
    };
  };

  // Fallback to static plans if API is unavailable
  const loadFallbackPlans = () => {
    const fallbackPersonal: DynamicPricingCard[] = [
      {
        id: 1,
        title: 'Free Plan',
        price: '0 USD',
        features: [
          { text: 'View 3 rankings', included: true },
          { text: 'Basic analytics', included: true },
          { text: 'Community support', included: true },
          { text: 'Public data access', included: true },
        ],
        buttonText: 'Start Free',
        buttonVariant: 'outline',
        promotions: [],
        badges: [],
      },
      {
        id: 2,
        title: 'Professional Plan',
        price: '49.99 USD',
        originalPrice: '59.99 USD',
        features: [
          { text: 'View 50 rankings', included: true },
          { text: 'Advanced analytics', included: true },
          { text: 'Priority support', included: true },
          { text: 'Custom reports', included: true },
          { text: 'Real-time data', included: true },
          { text: 'Export capabilities', included: true },
        ],
        highlight: true,
        buttonText: 'Most Popular',
        promotions: ['Limited Time Offer'],
        badges: ['POPULAR'],
        savings: 'Save USD 10.00',
      },
      {
        id: 3,
        title: 'Enterprise Plan',
        price: '199.99 USD',
        features: [
          { text: 'Unlimited rankings', included: true },
          { text: 'Full platform access', included: true },
          { text: 'Dedicated support', included: true },
          { text: 'Custom integrations', included: true },
          { text: 'White-label options', included: true },
          { text: 'SLA guarantee', included: true },
        ],
        buttonText: 'Contact Sales',
        promotions: [],
        badges: ['ENTERPRISE'],
      },
    ];

    const fallbackApi: DynamicPricingCard[] = [
      {
        id: 4,
        title: 'API Basic',
        price: '29.99 USD',
        features: [
          { text: '1,000 API calls/month', included: true },
          { text: 'Basic endpoints', included: true },
          { text: 'Documentation access', included: true },
          { text: 'Email support', included: true },
        ],
        buttonText: 'Get Started',
        promotions: [],
        badges: [],
      },
      {
        id: 5,
        title: 'API Pro',
        price: '99.99 USD',
        features: [
          { text: '10,000 API calls/month', included: true },
          { text: 'All endpoints', included: true },
          { text: 'Webhook support', included: true },
          { text: 'Priority support', included: true },
          { text: 'Rate limit increase', included: true },
        ],
        highlight: true,
        buttonText: 'Popular Choice',
        promotions: [],
        badges: ['RECOMMENDED'],
      },
      {
        id: 6,
        title: 'API Enterprise',
        price: '299.99 USD',
        features: [
          { text: 'Unlimited API calls', included: true },
          { text: 'Custom endpoints', included: true },
          { text: 'SLA guarantee', included: true },
          { text: 'Dedicated support', included: true },
          { text: 'Custom rate limits', included: true },
        ],
        buttonText: 'Contact Sales',
        promotions: [],
        badges: ['ENTERPRISE'],
      },
    ];

    setPersonalPlans(fallbackPersonal);
    setApiPlans(fallbackApi);
  };

  const handlePlanClick = (plan: DynamicPricingCard) => {
    // Build payment URL with affiliate tracking
    let paymentUrl = '/payment';
    const params = new URLSearchParams();
    
    if (affiliateCode) {
      params.set('ref', affiliateCode);
    }
    
    params.set('plan', plan.id.toString());
    
    if (params.toString()) {
      paymentUrl += `?${params.toString()}`;
    }
    
    router.push(paymentUrl);
  };


  const renderPricingCards = (cards: DynamicPricingCard[]) => {
    if (loading) {
      return Array.from({ length: 3 }).map((_, index) => (
        <div key={index} className="card-insight">
          <div className="h-6 bg-gray-300 rounded mb-4"></div>
          <div className="h-8 bg-gray-300 rounded mb-4"></div>
          <div className="space-y-2 mb-6">
            <div className="h-4 bg-gray-300 rounded"></div>
            <div className="h-4 bg-gray-300 rounded"></div>
            <div className="h-4 bg-gray-300 rounded"></div>
          </div>
          <div className="h-12 bg-gray-300 rounded"></div>
        </div>
      ));
    }

    return cards.map((card) => (
      <div key={card.id} className="relative">

        {/* Main Card */}
        <div
          className={`card-insight group relative overflow-visible h-full flex flex-col ${
            card.highlight
              ? 'insight-gradient-soft-highlight ring-2 ring-orange-200/60 border-orange-200/50 dark:border-orange-400/30 shadow-2xl shadow-orange-500/25'
              : 'ring-2 ring-blue-200/60 border-blue-200/50 dark:border-blue-400/30 shadow-xl shadow-blue-500/20'
          }`}
        >

          {/* Card Content - Normal padding */}
          <div className="relative px-6 sm:px-8 pt-6 sm:pt-8 pb-6 sm:pb-8 flex flex-col h-full">
            {/* Enhanced Title Section - Exact height for perfect alignment */}
            <div className="mb-4 h-[160px] flex flex-col">
              <div className={`${card.highlight ? 'h-[80px]' : 'h-[40px]'} flex flex-col justify-start mb-2`}>
                <h3 className={`text-xl sm:text-2xl font-bold leading-tight ${
                  card.highlight 
                    ? 'bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent' 
                    : 'text-foreground'
                } uppercase`}>
                  {card.title}
                </h3>
                {card.highlight && (
                  <div className="mt-2 inline-flex">
                    <div className="bg-gradient-to-r from-orange-500 to-yellow-500 text-white px-3 py-1 rounded-full text-xs font-bold tracking-wide border-2 border-orange-300/50 shadow-lg shadow-orange-500/30">
                      ⭐ MOST POPULAR ⭐
                    </div>
                  </div>
                )}
              </div>
              
              {/* Enhanced Price Display - Exact height for perfect alignment */}
              <div className={`${card.highlight ? 'h-[58px]' : 'h-[78px]'} flex flex-col justify-center`}>
                <div className="flex items-baseline gap-3">
                  <span className={`text-4xl sm:text-5xl font-bold leading-none ${
                    card.highlight 
                      ? 'bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent'
                      : 'insight-gradient-text'
                  }`}>
                    {card.price}
                  </span>
                  {card.originalPrice && (
                    <span className="text-lg text-gray-400 line-through decoration-2">
                      {card.originalPrice}
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Enhanced Features List - Fixed minimum height for alignment */}
            <div className="space-y-4 mb-8 flex-grow min-h-[200px] flex flex-col">
              {card.features.map((feature, idx) => (
                <div key={idx} className="flex items-start group/feature">
                  <div className={`flex-shrink-0 p-1.5 rounded-full ${
                    card.highlight
                      ? 'bg-orange-100 dark:bg-orange-900/30'
                      : 'bg-insight-primary/20'
                  }`}>
                    <Check className={`h-4 w-4 ${
                      card.highlight ? 'text-orange-600 dark:text-orange-400' : 'text-insight-primary'
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
                  <Star className="h-4 w-4" />
                  <span className="font-semibold">Affiliate Bonus:</span>
                  <span>{affiliateInfo.commission_rate}% commission applied</span>
                </div>
              </div>
            )}

            {/* Enhanced Action Button - Always at bottom */}
            <div className="mt-auto">
              <button
                className={`relative w-full rounded-xl font-semibold text-base py-4 overflow-hidden group ${
                  card.highlight
                    ? 'bg-gradient-to-r from-orange-400 via-amber-400 via-yellow-400 via-amber-500 to-orange-500 hover:from-orange-500 hover:via-amber-500 hover:via-yellow-500 hover:via-amber-600 hover:to-orange-600 text-white shadow-xl shadow-orange-500/40 border-0'
                    : 'bg-gradient-to-r from-blue-400 via-cyan-400 via-blue-300 via-cyan-400 to-blue-400 hover:from-blue-500 hover:via-cyan-500 hover:via-blue-400 hover:via-cyan-500 hover:to-blue-500 text-white shadow-lg shadow-blue-400/30 border-0'
                } before:absolute before:inset-0 before:bg-gradient-to-r before:from-transparent before:via-white/10 before:to-transparent before:translate-x-[-100%] hover:before:translate-x-[100%] before:transition-transform before:duration-700`}
                onClick={() => handlePlanClick(card)}
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
    ));
  };

  return (
    <div className="relative w-full py-16 sm:py-24 lg:py-32 overflow-hidden">
      {/* Analytics-style background decorations */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full animate-float" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full animate-bounce-gentle" />
        <div className="absolute top-1/2 left-1/4 w-16 h-16 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full animate-pulse-gentle" />
        <div className="absolute bottom-1/4 right-1/3 w-20 h-20 bg-gradient-to-br from-green-400/10 to-emerald-400/10 rounded-full animate-float-reverse" />
      </div>

      {/* Affiliate attribution banner */}
      {affiliateCode && affiliateInfo && (
        <div className="container mx-auto px-4 mb-8">
          <div className="bg-gradient-to-r from-purple-500 to-pink-500 text-white p-4 rounded-2xl text-center shadow-xl">
            <div className="flex items-center justify-center gap-2 text-lg font-semibold">
              <Star className="h-5 w-5" />
              <span>You're eligible for {affiliateInfo.commission_rate}% affiliate rewards!</span>
              <Star className="h-5 w-5" />
            </div>
            <p className="text-sm mt-1 opacity-90">
              Referred by partner: {affiliateCode} • Special pricing applied
            </p>
          </div>
        </div>
      )}

      <div className="relative container mx-auto px-4 sm:px-6 lg:px-8 max-w-7xl space-y-16 sm:space-y-20 lg:space-y-24">
        
        {/* Enhanced Promotional Banner */}
        <div className="text-center space-y-6 animate-fade-in">
          <div className="inline-flex items-center gap-2 bg-gradient-to-r from-orange-500 to-pink-500 text-white px-6 py-3 rounded-full text-sm font-bold shadow-lg">
            <Sparkles className="h-4 w-4 animate-spin" />
            <span>LIMITED TIME OFFER</span>
            <Sparkles className="h-4 w-4 animate-pulse" />
          </div>
          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold">
            <span className="bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
              Choose Your Plan
            </span>
          </h1>
          <p className="text-xl sm:text-2xl text-gray-600 dark:text-gray-300 max-w-4xl mx-auto">
            Unlock the power of advanced analytics with our flexible pricing options designed for every need
          </p>
        </div>
        {/* Personal Plans */}
        <div className="space-y-8 sm:space-y-12">
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

            {/* Decorative elements */}
            <div className="flex justify-center items-center gap-4 mt-6">
              <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
              <div
                className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse"
                style={{ animationDelay: '0.5s' }}
              />
              <div
                className="w-2 h-2 bg-orange-400 rounded-full animate-pulse"
                style={{ animationDelay: '1s' }}
              />
            </div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-12 lg:gap-16 px-4 py-8">
            {renderPricingCards(personalPlans)}
          </div>
        </div>

        {/* API Plans */}
        <div className="space-y-8 sm:space-y-12">
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
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-12 lg:gap-16 px-4 py-8">
            {renderPricingCards(apiPlans)}
          </div>
        </div>
      </div>
    </div>
  );
};

export default DynamicPricingSection;