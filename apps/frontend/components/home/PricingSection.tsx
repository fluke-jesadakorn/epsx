'use client';

import { Check, Sparkles } from 'lucide-react';
import React from 'react';
import { PACKAGES } from '@/app/constants/packages';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';

interface PlanFeature {
  text: string;
  included: boolean;
}

interface PricingCard {
  title: string;
  price: string;
  features: PlanFeature[];
  highlight?: boolean;
  buttonText: string;
  buttonVariant?: 'default' | 'outline';
}

type ApiPlan = {
  title: string;
  price: string;
  features: PlanFeature[];
  highlight?: boolean;
  buttonText: string;
  buttonVariant?: 'default' | 'outline';
};

const apiPlans: ApiPlan[] = [
  {
    title: 'API Personal',
    price: '999 USDT/M',
    features: [
      { text: '25 Data sets', included: true },
      { text: 'Country Selection', included: true },
      { text: 'Unlimited Accounts', included: true },
    ],
    buttonText: 'Get Started',
  },
  {
    title: 'API Company',
    price: '2,999 USDT/M',
    features: [
      { text: '100 Data sets', included: true },
      { text: 'Country Selection', included: true },
      { text: 'Unlimited Accounts', included: true },
    ],
    buttonText: 'Get Started',
    highlight: true,
  },
  {
    title: 'API Partner',
    price: 'Revenue Share',
    features: [
      { text: '100 Data sets', included: true },
      { text: 'Country Selection', included: true },
      { text: 'Industry Selection', included: true },
      { text: '15% Revenue Share/Transaction', included: true },
      { text: 'Unlimited Accounts', included: true },
    ],
    buttonText: 'Partner With Us',
    buttonVariant: 'outline' as const,
  },
];

const packageToPricingCard = (pkg: any): PricingCard => ({
  title: pkg.name,
  price: `${pkg.price} ${pkg.currency}`,
  features: pkg.features.map((feature: string) => ({
    text: feature,
    included: true,
  })),
  highlight: pkg.displayTier === 'GOLD',
  buttonText: pkg.price === 0 ? 'Start Trial' : 'Subscribe Now',
  buttonVariant: pkg.price === 0 ? 'outline' : 'default',
});

const PricingSection = () => {
  const personalPlans = PACKAGES.filter((pkg) =>
    ['BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'].includes(pkg.displayTier),
  ).map(packageToPricingCard);


  const router = useRouter();

  const renderPricingCards = (cards: PricingCard[]) => {
    return cards.map((card, index) => (
      <div
        key={index}
        className={`card-insight group relative ${
          card.highlight
            ? 'ring-2 ring-blue-400/30 shadow-xl border-blue-200/40 dark:border-blue-400/30'
            : 'border border-gray-200/20 dark:border-gray-700/20'
        }`}
      >
        <div className="p-6">
          {/* Clean title with integrated badge */}
          <div className="flex justify-between items-start mb-6">
            <h3 className="text-xl font-bold text-foreground uppercase">
              {card.title}
              {card.highlight && (
                <span className="text-sm font-normal text-muted-foreground ml-2">
                  • MOST POPULAR
                </span>
              )}
            </h3>
          </div>

          {/* Price section with cleaner layout */}
          <div className="mb-6">
            <div className="text-3xl font-bold text-foreground mb-1">
              {card.title === 'Gold Plan' ? '6.93 USDT/month' : card.price}
            </div>
            <div className="border-b-2 border-gray-300 dark:border-gray-600 w-32 mb-2"></div>
            {card.title === 'Gold Plan' && (
              <div className="text-sm text-muted-foreground">
                was 9.90 • Save 30%
              </div>
            )}
          </div>

          {/* What's included section */}
          <div className="mb-6">
            <div className="text-sm font-semibold text-foreground mb-3">
              What's included:
            </div>
            <ul className="space-y-2">
              {card.features.map((feature, idx) => (
                <li key={idx} className="flex items-start text-sm text-muted-foreground">
                  <span className="mr-2">•</span>
                  {feature.text}
                </li>
              ))}
            </ul>
          </div>

          {/* Clean CTA button */}
          <Button
            variant={card.highlight ? 'default' : 'outline'}
            size="lg"
            className={`w-full font-semibold ${
              card.highlight 
                ? 'bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white' 
                : ''
            }`}
            onClick={() => router.push('/payment')}
          >
            {card.title === 'Gold Plan' ? 'START NOW 🚀' : card.buttonText}
          </Button>

          {/* Money back guarantee */}
          {card.highlight && (
            <div className="text-center text-sm text-muted-foreground mt-4">
              💰 30-day money back guarantee
            </div>
          )}
        </div>
      </div>
    ));
  };

  return (
    <div className="relative w-full py-16 sm:py-24 lg:py-32">
      <div className="relative container mx-auto px-4 sm:px-6 lg:px-8 max-w-7xl space-y-16 sm:space-y-24 lg:space-y-32">
        {/* Personal Plans */}
        <div className="space-y-8 sm:space-y-12">
          <div className="text-center space-y-6 sm:space-y-8">
            <h2 className="text-4xl sm:text-6xl font-bold">
              <span className="mr-2">💰</span>
              <span className="bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent">
                Personal Plans
              </span>
            </h2>
            <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
              🚀 Choose the perfect plan for individual use and start your
              data journey
            </p>
            <div className="w-32 sm:w-40 h-1.5 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 mx-auto rounded-full" />
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8 lg:gap-12">
            {renderPricingCards(personalPlans)}
          </div>
        </div>

        {/* API Plans */}
        <div className="space-y-8 sm:space-y-12">
          <div className="text-center space-y-4 sm:space-y-6">
            <h2 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-orange-500 via-pink-500 to-purple-500 bg-clip-text text-transparent">
              API Plans
            </h2>
            <p className="text-lg sm:text-xl text-foreground/80 max-w-2xl mx-auto">
              Integrate our powerful API into your systems
            </p>
            <div className="w-24 sm:w-32 h-1 sm:h-1.5 bg-gradient-to-r from-orange-500 to-pink-500 mx-auto rounded-full" />
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 sm:gap-8 lg:gap-12">
            {renderPricingCards(apiPlans)}
          </div>
        </div>
      </div>
    </div>
  );
};

export default PricingSection;
