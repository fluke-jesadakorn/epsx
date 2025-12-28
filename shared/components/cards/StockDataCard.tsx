'use client';

import { cn } from '../../utils';

// ============================================================================
// STOCK DATA CARD TYPES
// ============================================================================

export interface StockDataCardProps {
  symbol: string;
  rank: number;
  epsGrowth: number;
  price: number;
  currency?: string;
  daysUntilNextAction?: number;
  variant?: 'premium' | 'standard';
  className?: string;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

const formatPercentage = (value: number): string => {
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

const formatCurrency = (value: number, currency: string = 'USD'): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
};

const getRankBadge = (rank: number): { badge: string; color: string } => {
  if (rank === 1) return { badge: '👑', color: 'from-yellow-400 to-orange-500' };
  if (rank === 2) return { badge: '🥈', color: 'from-slate-400 to-slate-500' };
  if (rank === 3) return { badge: '🥉', color: 'from-orange-400 to-amber-500' };
  if (rank === 4) return { badge: '⭐', color: 'from-purple-400 to-pink-500' };
  if (rank === 5) return { badge: '💎', color: 'from-blue-400 to-cyan-500' };
  return { badge: '🔥', color: 'from-green-400 to-emerald-500' };
};

const getRankTitle = (rank: number): string => {
  if (rank === 1) return '👑 CHAMPION';
  if (rank === 2) return '🥈 ELITE';
  if (rank === 3) return '🥉 LEGEND';
  if (rank === 4) return '⭐ MASTER';
  if (rank === 5) return '💎 DIAMOND';
  return '🔥 LEADER';
};

const getPremiumStyle = (_rank: number) => {
  // All ranks use the same standard dark background styling (matching StandardStockCard)
  return {
    container: 'bg-gradient-to-br from-white via-slate-50 to-gray-100 dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 border-2 border-slate-300 dark:border-slate-600 shadow-2xl',
    glow: '',
  };
};

// ============================================================================
// NEXT ACTION PROGRESS COMPONENT
// ============================================================================

const NextActionProgress = ({ daysUntilNextAction }: { daysUntilNextAction: number }) => {
  const progressPercentage = Math.max(0, Math.min(100, ((90 - daysUntilNextAction) / 90) * 100));

  return (
    <div className="mb-6">
      <div className="mb-2 flex items-center justify-between px-1">
        <div className="text-xs font-light tracking-wider text-slate-400 dark:text-slate-300 uppercase">
          Next Action
        </div>
        <div className="text-lg font-bold text-green-700 dark:text-green-400">
          {daysUntilNextAction}d
        </div>
      </div>

      {/* Progress indicator */}
      <div className="relative h-2 overflow-hidden rounded-full bg-gradient-to-r from-slate-200 via-slate-100 to-slate-200 dark:from-slate-600 dark:via-slate-500 dark:to-slate-600">
        <div className="absolute inset-0 bg-gradient-to-r from-white/50 dark:from-slate-300/30 via-transparent to-white/50 dark:to-slate-300/30" />
        <div
          className="relative h-full overflow-hidden rounded-full bg-gradient-to-r from-green-500 via-emerald-400 to-green-600"
          style={{ width: `${progressPercentage}%` }}
        >
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent" />
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// PREMIUM STOCK CARD (for top ranks)
// ============================================================================

const PremiumStockCard = ({
  symbol,
  rank,
  epsGrowth,
  price,
  currency = 'USD',
  daysUntilNextAction,
  className,
}: StockDataCardProps) => {
  const rankInfo = getRankBadge(rank);
  const premiumStyle = getPremiumStyle(rank);

  return (
    <div
      className={cn(
        'relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px]',
        premiumStyle.container,
        premiumStyle.glow,
        className
      )}
    >
      {/* Floating rank badge */}
      <div
        className={cn(
          'absolute -top-4 -left-4 h-16 w-16 flex rotate-12 transform items-center justify-center rounded-full border-4 border-white text-2xl text-white shadow-2xl z-30',
          `bg-gradient-to-br ${rankInfo.color}`
        )}
      >
        {rankInfo.badge}
      </div>

      {/* Corner effects */}
      <div className="bg-gradient-radial absolute top-0 right-0 h-20 w-20 rounded-bl-3xl from-white/40 dark:from-white/10 via-transparent to-transparent opacity-60" />
      <div className="bg-gradient-radial absolute bottom-0 left-0 h-20 w-20 rounded-tr-3xl from-white/40 dark:from-white/10 via-transparent to-transparent opacity-60" />

      <div className="p-8 pt-16">
        {/* Header */}
        <div className="mb-6 text-center">
          <div className="mb-3">
            <div className="mb-1 text-xs font-light tracking-[0.2em] text-slate-400 uppercase">
              {getRankTitle(rank)} · RANK #{rank}
            </div>
            <h3 className="mb-2 bg-gradient-to-r from-slate-700 via-slate-900 to-slate-700 dark:from-slate-200 dark:via-white dark:to-slate-200 bg-clip-text text-2xl font-black tracking-tight text-transparent sm:text-3xl">
              {symbol}
            </h3>
            <div className="mx-auto h-px w-16 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent" />
          </div>

          {/* Next Action Progress */}
          {daysUntilNextAction !== undefined && (
            <NextActionProgress daysUntilNextAction={daysUntilNextAction} />
          )}

          {/* View Details Button */}
          <div className="flex justify-center">
            <a
              href={`https://www.tradingview.com/symbols/${symbol}`}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-bold text-white shadow-xl bg-gradient-to-r from-green-600 via-green-500 to-emerald-600"
            >
              <span className="text-base sm:text-lg">📊</span>
              <span className="tracking-wide text-sm sm:text-base">VIEW DETAILS</span>
              <span className="text-xs opacity-75">→</span>
            </a>
          </div>
        </div>



        {/* Data Display */}
        <div className="flex flex-col gap-4">
          {/* Growth */}
          <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
            <div className="mb-4 flex items-center justify-center gap-3">
              <span className="text-xl sm:text-2xl">{epsGrowth >= 0 ? '📈' : '📉'}</span>
              <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">Growth</div>
            </div>
            <div
              className={cn(
                'mb-2 text-center text-lg leading-tight font-black sm:text-2xl',
                epsGrowth >= 0 ? 'text-green-700 dark:text-green-400' : 'text-red-700 dark:text-red-400'
              )}
            >
              {formatPercentage(epsGrowth)}
            </div>
            <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent" />
          </div>

          {/* Price */}
          <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
            <div className="mb-4 flex items-center justify-center gap-3">
              <span className="text-xl sm:text-2xl">💰</span>
              <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">Price</div>
            </div>
            <div className="mb-2 text-center text-lg leading-tight font-black text-slate-800 dark:text-slate-200 sm:text-2xl">
              {formatCurrency(price, currency)}
            </div>
            <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent" />
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// STANDARD STOCK CARD (for regular ranks)
// ============================================================================

const StandardStockCard = ({
  symbol,
  rank,
  epsGrowth,
  price,
  currency = 'USD',
  daysUntilNextAction,
  className,
}: StockDataCardProps) => {
  const progressPercentage = daysUntilNextAction !== undefined
    ? Math.max(0, Math.min(100, ((90 - daysUntilNextAction) / 90) * 100))
    : 0;

  return (
    <div
      className={cn(
        'relative w-full max-w-[320px] min-w-[240px] flex-shrink-0 overflow-hidden rounded-3xl border-2 bg-gradient-to-br from-white via-slate-50 to-gray-100 dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 shadow-2xl sm:min-w-[280px]',
        'border-slate-300 dark:border-slate-600',
        className
      )}
    >
      {/* Corner accent */}
      <div className="absolute top-0 right-0 h-16 w-16 bg-gradient-to-bl from-slate-400/20 dark:from-slate-500/20 to-transparent rounded-bl-3xl z-[-1]" />

      <div className="p-6">
        {/* Header */}
        <div className="mb-4">
          <div className="mb-2 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="text-center">
                <div className="text-xs font-medium tracking-wide text-slate-500 dark:text-slate-400 uppercase">
                  Rank #{rank}
                </div>
                <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200 sm:text-xl">
                  {symbol}
                </h3>
              </div>
            </div>
            <a
              href={`https://www.tradingview.com/symbols/${symbol}`}
              target="_blank"
              rel="noopener noreferrer"
              className="rounded-full px-4 py-2 text-xs font-semibold text-white shadow-lg bg-gradient-to-r from-slate-500 to-slate-600"
            >
              📊 View
            </a>
          </div>
        </div>

        {/* Next Action Progress */}
        {daysUntilNextAction !== undefined && (
          <div className="mb-4">
            <div className="mb-3 flex items-center justify-between">
              <div className="text-xs text-slate-500 dark:text-slate-400">Next Action</div>
              <div className="text-sm font-bold text-green-600 dark:text-green-400">
                {daysUntilNextAction}d
              </div>
            </div>
            <div className="h-1 rounded-full bg-slate-200 dark:bg-slate-600">
              <div
                className="h-full rounded-full bg-gradient-to-r from-green-400 to-emerald-500"
                style={{ width: `${progressPercentage}%` }}
              />
            </div>
          </div>
        )}

        {/* Data Display */}
        <div className="flex flex-col gap-3">
          {/* Growth */}
          <div className="rounded-2xl border border-slate-200/50 dark:border-slate-600/50 bg-white/50 dark:bg-gray-800/50 p-4 text-center backdrop-blur-sm">
            <div className="mb-2 flex items-center justify-center gap-2">
              <span className="text-base sm:text-lg">{epsGrowth >= 0 ? '📈' : '📉'}</span>
              <span className="text-sm font-medium text-slate-600 dark:text-slate-400">Growth</span>
            </div>
            <div
              className={cn(
                'text-center text-base leading-tight font-bold sm:text-xl',
                epsGrowth >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
              )}
            >
              {formatPercentage(epsGrowth)}
            </div>
          </div>

          {/* Price */}
          <div className="rounded-2xl border border-slate-200/50 dark:border-slate-600/50 bg-white/50 dark:bg-gray-800/50 p-4 text-center backdrop-blur-sm">
            <div className="mb-2 flex items-center justify-center gap-2">
              <span className="text-base sm:text-lg">💰</span>
              <span className="text-sm font-medium text-slate-600 dark:text-slate-400">Price</span>
            </div>
            <div className="text-center text-base leading-tight font-bold text-slate-800 dark:text-slate-200 sm:text-xl">
              {formatCurrency(price, currency)}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// MAIN STOCK DATA CARD COMPONENT
// ============================================================================

export const StockDataCard = ({
  variant = 'standard',
  ...props
}: StockDataCardProps) => {
  if (variant === 'premium') {
    return <PremiumStockCard {...props} />;
  }
  return <StandardStockCard {...props} />;
};

StockDataCard.displayName = 'StockDataCard';

export default StockDataCard;
