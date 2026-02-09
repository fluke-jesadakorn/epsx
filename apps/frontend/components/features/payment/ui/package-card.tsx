import { cn } from '@/lib/utils';
import { Check, Sparkles } from 'lucide-react';
import type { PaymentPackage } from '../types';

interface PackageCardProps {
  pkg: PaymentPackage;
  isSelected: boolean;
  onSelect: () => void;
  isMobile?: boolean;
}

export function PackageCard({ pkg, isSelected, onSelect, isMobile = false }: PackageCardProps) {
  const cardWrapperClass = isMobile
    ? 'relative w-80 flex-shrink-0'
    : 'relative';

  const containerClass = isMobile
    ? 'overflow-x-auto pb-4'
    : 'hidden gap-6 md:grid md:grid-cols-3';

  const gridClass = isMobile
    ? 'flex gap-4 px-2'
    : 'mb-8 gap-6';

  if (isMobile) {
    return (
      <div key={`${pkg.id} - ${pkg.name}`} className={cardWrapperClass}>
        <div
          className={cn(
            'card-insight group relative flex h-full cursor-pointer flex-col overflow-visible',
            isSelected
              ? pkg.popular
                ? 'border-orange-200/50 shadow-2xl ring-2 shadow-orange-500/25 ring-orange-200/60 dark:border-orange-400/30'
                : 'border-blue-200/50 shadow-xl ring-2 shadow-blue-500/20 ring-blue-200/60 dark:border-blue-400/30'
              : pkg.popular
                ? 'border-orange-200/50 shadow-xl ring-2 shadow-orange-500/20 ring-orange-200/60 dark:border-orange-400/30'
                : 'border-blue-200/50 shadow-lg ring-2 shadow-blue-500/15 ring-blue-200/60 dark:border-blue-400/30'
          )}
          onClick={onSelect}
        >
          <PackageCardContent pkg={pkg} isSelected={isSelected} />
          <div className="absolute -right-2 -bottom-2 h-12 w-12 rounded-full bg-gradient-to-br from-transparent via-transparent to-gray-100/30 blur-xl dark:to-gray-800/30" />
          <div className="absolute -top-2 -left-2 h-8 w-8 rounded-full bg-gradient-to-br from-transparent via-transparent to-blue-100/20 blur-lg dark:to-blue-800/20" />
        </div>
      </div>
    );
  }

  return (
    <div key={`${pkg.id} - ${pkg.name}`} className="relative">
      <div
        className={cn(
          'card-insight group relative flex h-full cursor-pointer flex-col overflow-visible',
          isSelected
            ? pkg.popular
              ? 'border-orange-200/50 shadow-2xl ring-2 shadow-orange-500/25 ring-orange-200/60 dark:border-orange-400/30'
              : 'border-blue-200/50 shadow-xl ring-2 shadow-blue-500/20 ring-blue-200/60 dark:border-blue-400/30'
            : pkg.popular
              ? 'border-orange-200/50 shadow-xl ring-2 shadow-orange-500/20 ring-orange-200/60 dark:border-orange-400/30'
              : 'border-blue-200/50 shadow-lg ring-2 shadow-blue-500/15 ring-blue-200/60 dark:border-blue-400/30'
        )}
        onClick={onSelect}
      >
        <PackageCardContent pkg={pkg} isSelected={isSelected} isDesktop />
        <div className="absolute -right-2 -bottom-2 h-12 w-12 rounded-full bg-gradient-to-br from-transparent via-transparent to-gray-100/30 blur-xl dark:to-gray-800/30" />
        <div className="absolute -top-2 -left-2 h-8 w-8 rounded-full bg-gradient-to-br from-transparent via-transparent to-blue-100/20 blur-lg dark:to-blue-800/20" />
      </div>
    </div>
  );
}

interface PackageCardContentProps {
  pkg: PaymentPackage;
  isSelected: boolean;
  isDesktop?: boolean;
}

function PackageCardContent({ pkg, isSelected, isDesktop = false }: PackageCardContentProps) {
  const paddingClass = isDesktop ? 'px-6 pt-6 pb-6 sm:px-8 sm:pt-8 sm:pb-8' : 'px-6 pt-6 pb-6';
  const titleSize = isDesktop ? 'sm:text-2xl' : 'text-xl';
  const featureSize = isDesktop ? 'sm:text-base' : 'text-sm';

  return (
    <div className={`relative flex h-full flex-col ${paddingClass}`}>
      <div className="mb-4 flex h-[160px] flex-col items-center text-center">
        <div
          className={cn(
            pkg.popular ? 'h-[80px]' : 'h-[40px]',
            'mb-2 flex flex-col items-center justify-start'
          )}
        >
          <h3
            className={cn(
              `text-xl leading-tight font-bold whitespace-nowrap uppercase ${titleSize}`,
              pkg.popular
                ? 'bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent'
                : 'text-foreground'
            )}
          >
            {pkg.name}
          </h3>
          {pkg.popular && (
            <div className="mt-2">
              <div className="rounded-full border-2 border-orange-300/50 bg-gradient-to-r from-orange-500 to-yellow-500 px-3 py-1 text-xs font-bold tracking-wide text-white shadow-lg shadow-orange-500/30">
                ⭐ MOST POPULAR ⭐
              </div>
            </div>
          )}
        </div>

        <div
          className={cn(
            pkg.popular ? 'h-[58px]' : 'h-[78px]',
            'flex flex-col items-center justify-center'
          )}
        >
          <div className="flex flex-wrap items-baseline justify-center gap-3">
            <span
              className={cn(
                'text-4xl leading-none font-bold whitespace-nowrap',
                isDesktop ? 'sm:text-5xl' : '',
                pkg.popular
                  ? 'bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent'
                  : 'insight-gradient-text'
              )}
            >
              ${pkg.current_price}
            </span>
            {pkg.base_price > pkg.current_price && (
              <span className="text-lg whitespace-nowrap text-gray-400 line-through decoration-2">
                ${pkg.base_price}
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="mb-8 flex min-h-[200px] flex-grow flex-col space-y-4">
        {pkg.features.map((feature) => (
          <div key={feature} className="group/feature flex items-start">
            <div
              className={cn(
                'flex-shrink-0 rounded-full p-1.5',
                pkg.popular
                  ? 'bg-orange-100 dark:bg-orange-900/30'
                  : 'bg-insight-primary/20'
              )}
            >
              <Check
                className={cn(
                  'h-4 w-4',
                  pkg.popular
                    ? 'text-orange-600 dark:text-orange-400'
                    : 'text-insight-primary'
                )}
              />
            </div>
            <span className={cn('text-muted-foreground ml-3 text-sm font-medium', featureSize)}>
              {feature}
            </span>
          </div>
        ))}
      </div>

      <div className="mt-auto">
        <button
          className={cn(
            'group relative w-full overflow-hidden rounded-xl py-4 text-base font-semibold',
            pkg.popular
              ? 'border-0 bg-gradient-to-r from-orange-400 via-amber-400 via-amber-500 via-yellow-400 to-orange-500 text-white shadow-xl shadow-orange-500/40'
              : 'border-0 bg-gradient-to-r from-blue-400 via-blue-300 via-cyan-400 to-blue-400 text-white shadow-lg shadow-blue-400/30'
          )}
        >
          <span className="relative flex items-center justify-center gap-2">
            {isSelected ? 'Selected' : 'Select Plan'}
            {pkg.popular && <Sparkles className="h-4 w-4" />}
          </span>
        </button>
      </div>
    </div>
  );
}
