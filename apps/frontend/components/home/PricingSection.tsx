'use client';

import { Check, Sparkles } from 'lucide-react';
import React from 'react';
import { PACKAGES, LEVEL_BENEFITS } from '@/app/constants/packages';
import type { Package } from '@/app/constants/packages';
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

const packageToPricingCard = (pkg: Package): PricingCard => ({
  title: pkg.name,
  price: `${pkg.price} ${pkg.currency}`,
  features: LEVEL_BENEFITS[pkg.level].map((benefit) => ({
    text: benefit,
    included: true,
  })),
  highlight: pkg.level === 'GOLD' || pkg.level === 'PLATINUM',
  buttonText: pkg.price === 0 ? 'Start Trial' : 'Subscribe Now',
  buttonVariant: pkg.price === 0 ? 'outline' : 'default',
});

const PricingSection = () => {
  const personalPlans = PACKAGES.filter((pkg) =>
    ['BASIC', 'SILVER', 'GOLD'].includes(pkg.level),
  ).map(packageToPricingCard);

  const companyPlans = PACKAGES.filter((pkg) => pkg.level === 'PLATINUM').map(
    packageToPricingCard,
  );

  const router = useRouter();

  const renderPricingCards = (cards: PricingCard[]) => {
    return cards.map((card, index) => (
      <div
        key={index}
        className={`card-insight group relative transition-all duration-300 ${
          card.highlight
            ? 'insight-gradient-soft-highlight ring-2 ring-soft-blue scale-105 border-blue-200/40 dark:border-blue-400/30 shadow-xl shadow-blue-500/20'
            : 'hover:insight-shadow hover:scale-[1.02] hover:shadow-blue-300/20'
        }`}
      >
        {/* Analytics-style decorative elements */}
        {card.highlight && (
          <>
            <div className="absolute -top-4 left-1/2 transform -translate-x-1/2">
              <div className="bg-gradient-to-r from-orange-500 to-yellow-500 text-white px-4 py-1 rounded-full text-sm font-bold flex items-center gap-1">
                <span className="animate-bounce">🔥</span>
                MOST POPULAR
                <span className="animate-pulse">✨</span>
              </div>
            </div>
            <div className="absolute top-2 right-2 text-2xl opacity-20 animate-spin-slow">
              📊
            </div>
          </>
        )}

        {/* Corner decorations for all cards */}
        <div className="absolute bottom-2 left-2 text-lg opacity-15 group-hover:opacity-30 transition-opacity duration-300 animate-pulse">
          📈
        </div>
        {/* Decorative elements */}
        <div
          className={`absolute -top-4 -left-4 w-8 h-8 rounded-full blur-lg transition-colors duration-300 ${
            card.highlight
              ? 'bg-orange-300/30 group-hover:bg-orange-400/40'
              : 'bg-insight-primary/20 group-hover:bg-insight-primary/40'
          }`}
        />
        <div
          className={`absolute -bottom-4 -right-4 w-8 h-8 rounded-full blur-lg transition-colors duration-300 ${
            card.highlight
              ? 'bg-amber-300/30 group-hover:bg-amber-400/40'
              : 'bg-insight-secondary/20 group-hover:bg-insight-secondary/40'
          }`}
        />

        <div className="relative">
          {/* Title with sparkle effect */}
          <div className="flex items-center gap-2 mb-2">
            <h3 className="text-xl sm:text-2xl font-bold text-foreground">
              {card.title}
            </h3>
            {card.highlight && (
              <Sparkles className="h-5 w-5 text-insight-primary animate-bounce-gentle" />
            )}
          </div>

          {/* Price with enhanced styling */}
          <div className="mt-4 sm:mt-6 mb-6 sm:mb-8">
            <div className="flex items-baseline gap-2">
              <span className="text-4xl sm:text-5xl font-bold insight-gradient-text">
                {card.price}
              </span>
            </div>
          </div>

          {/* Features with enhanced styling */}
          <ul className="space-y-3 sm:space-y-4 mb-6 sm:mb-8">
            {card.features.map((feature, idx) => (
              <li key={idx} className="flex items-start group/feature">
                <div className="flex-shrink-0 p-1 rounded-full bg-insight-primary/20 group-hover/feature:bg-insight-primary/30 transition-colors duration-300">
                  <Check className="h-4 w-4 text-insight-primary" />
                </div>
                <span className="ml-3 text-sm sm:text-base text-muted-foreground group-hover/feature:text-foreground transition-colors">
                  {feature.text}
                </span>
              </li>
            ))}
          </ul>

          {/* Enhanced button */}
          <Button
            variant={
              card.highlight
                ? 'insight'
                : card.buttonVariant === 'outline'
                  ? 'insight-outline'
                  : 'insight-secondary'
            }
            size="lg"
            className="w-full rounded-2xl font-semibold"
            onClick={() => router.push('/payment')}
          >
            {card.buttonText}
          </Button>
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

      <div className="relative container mx-auto px-4 sm:px-6 lg:px-8 max-w-7xl space-y-16 sm:space-y-24 lg:space-y-32">
        {/* Personal Plans */}
        <div className="space-y-8 sm:space-y-12">
          <div className="text-center space-y-6 sm:space-y-8 animate-slide-up">
            <h2 className="text-4xl sm:text-6xl font-bold bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
              💰 Personal Plans
            </h2>
            <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
              🚀 Choose the perfect plan for individual use and start your
              data journey
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
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8 lg:gap-12 animate-slide-up-delayed">
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
