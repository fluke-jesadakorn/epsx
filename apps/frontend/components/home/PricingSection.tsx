'use client';

import { Check, Sparkles } from 'lucide-react';
import React from 'react';
import { Button } from '@/components/ui/button';
import { PACKAGES, LEVEL_BENEFITS } from '@/app/constants/packages';
import type { Package } from '@/app/constants/packages';

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

const packageToPricingCard = (pkg: Package): PricingCard => ({
  title: pkg.name,
  price: `${pkg.price} ${pkg.currency}`,
  features: LEVEL_BENEFITS[pkg.level].map(benefit => ({
    text: benefit,
    included: true
  })),
  highlight: pkg.level === 'GOLD' || pkg.level === 'PLATINUM',
  buttonText: pkg.price === 0 ? 'Start Trial' : 'Subscribe Now',
  buttonVariant: pkg.price === 0 ? 'outline' : 'default'
});

const PricingSection = () => {
  const personalPlans = PACKAGES.filter(pkg => 
    ['BASIC', 'SILVER', 'GOLD'].includes(pkg.level)
  ).map(packageToPricingCard);

  const companyPlans = PACKAGES.filter(pkg => 
    pkg.level === 'PLATINUM'
  ).map(packageToPricingCard);


  const renderPricingCards = (cards: PricingCard[]) => {
    return cards.map((card, index) => (
      <div
        key={index}
        className={`group relative rounded-3xl p-6 sm:p-8 backdrop-blur-sm border transition-all duration-300 hover:scale-105 hover:shadow-2xl ${
          card.highlight
            ? 'bg-gradient-to-b from-blue-500/30 to-purple-500/30 border-blue-500/40'
            : 'bg-card/80 border-border hover:border-blue-500/40'
        }`}
      >
        {/* Decorative elements */}
        <div className="absolute -top-6 -left-6 w-12 h-12 bg-blue-500/20 rounded-full blur-xl group-hover:bg-blue-500/30 transition-colors duration-300" />
        <div className="absolute -bottom-6 -right-6 w-12 h-12 bg-purple-500/20 rounded-full blur-xl group-hover:bg-purple-500/30 transition-colors duration-300" />

        <div className="relative">
          {/* Title with sparkle effect */}
          <div className="flex items-center gap-2">
            <h3 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
              {card.title}
            </h3>
            {card.highlight && (
              <Sparkles className="h-5 w-5 text-yellow-500 animate-pulse" />
            )}
          </div>

          {/* Price with enhanced styling */}
          <div className="mt-4 sm:mt-6 mb-6 sm:mb-8">
            <div className="flex items-baseline gap-2">
              <span className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
                {card.price}
              </span>
            </div>
          </div>

          {/* Features with enhanced styling */}
          <ul className="space-y-3 sm:space-y-4 mb-6 sm:mb-8">
            {card.features.map((feature, idx) => (
              <li key={idx} className="flex items-start group/feature">
                <div className="flex-shrink-0 p-1 rounded-full bg-gradient-to-r from-blue-500/20 to-purple-500/20 group-hover/feature:from-blue-500/30 group-hover/feature:to-purple-500/30">
                  <Check className="h-4 w-4 text-blue-500 group-hover/feature:text-blue-400" />
                </div>
                <span className="ml-3 text-sm sm:text-base text-foreground/75 group-hover/feature:text-foreground transition-colors">
                  {feature.text}
                </span>
              </li>
            ))}
          </ul>

          {/* Enhanced button */}
          <Button
            variant={card.buttonVariant || 'default'}
            className={`w-full h-10 sm:h-12 text-sm sm:text-base font-semibold transition-all duration-300 ${
              card.highlight
                ? 'bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white shadow-lg hover:shadow-blue-500/25'
                : 'hover:scale-105'
            }`}
            onClick={() => window.location.href = '/settings/payment'}
          >
            {card.buttonText}
          </Button>
        </div>
      </div>
    ));
  };

  return (
    <div className="w-full py-16 sm:py-24 lg:py-32 px-4 sm:px-6 overflow-hidden">
      <div className="relative max-w-7xl mx-auto space-y-16 sm:space-y-24 lg:space-y-32">
        {/* Personal Plans */}
        <div className="space-y-8 sm:space-y-12">
          <div className="text-center space-y-4 sm:space-y-6 animate-fade-in">
            <h2 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
              Personal Plans
            </h2>
            <p className="text-lg sm:text-xl text-foreground/80 max-w-2xl mx-auto">
              Choose the perfect plan for individual use
            </p>
            <div className="relative w-24 sm:w-32 h-1 sm:h-1.5 bg-gradient-to-r from-blue-500 to-purple-500 mx-auto rounded-full overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/25 to-transparent animate-shimmer" />
            </div>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 sm:gap-8 lg:gap-12 animate-fade-in-delayed">
            {renderPricingCards(personalPlans)}
          </div>
        </div>

        {/* Company Plans */}
        <div className="space-y-8 sm:space-y-12">
          <div className="text-center space-y-4 sm:space-y-6 animate-fade-in">
            <h2 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-cyan-500 via-blue-500 to-purple-500 bg-clip-text text-transparent">
              Company Plans
            </h2>
            <p className="text-lg sm:text-xl text-foreground/80 max-w-2xl mx-auto">
              Enterprise solutions for your business
            </p>
            <div className="relative w-24 sm:w-32 h-1 sm:h-1.5 bg-gradient-to-r from-cyan-500 to-blue-500 mx-auto rounded-full overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/25 to-transparent animate-shimmer" />
            </div>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 sm:gap-8 lg:gap-12 animate-fade-in-delayed">
            {renderPricingCards(companyPlans)}
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
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6 sm:gap-8 lg:gap-12 animate-fade-in-delayed">
            {renderPricingCards(apiPlans)}
          </div>
        </div>
      </div>
    </div>
  );
};

export default PricingSection;
