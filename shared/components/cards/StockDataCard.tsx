
import { ArrowRight, Calendar, TrendingUp } from 'lucide-react';
import { cn } from '../../utils';
import { PremiumCard } from '../ui/PremiumCard';

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

const getRankTheme = (rank: number): { color: string; glow: 'blue' | 'purple' | 'orange' | 'green' | 'none', label: string } => {
  if (rank === 1) return { color: 'text-yellow-400', glow: 'orange', label: 'CHAMPION' };
  if (rank === 2) return { color: 'text-slate-300', glow: 'blue', label: 'ELITE' };
  if (rank === 3) return { color: 'text-amber-500', glow: 'orange', label: 'LEGEND' };
  if (rank <= 5) return { color: 'text-blue-400', glow: 'blue', label: 'TOP 5' };
  return { color: 'text-blue-500', glow: 'none', label: `RANK #${rank}` };
};

// ============================================================================
// MAIN STOCK DATA CARD COMPONENT
// ============================================================================

export const StockDataCard = ({
  symbol,
  rank,
  epsGrowth,
  price,
  currency = 'USD',
  daysUntilNextAction,
  variant = 'standard',
  className,
}: StockDataCardProps) => {
  const rankTheme = getRankTheme(rank);
  const isPositiveGrowth = epsGrowth >= 0;

  return (
    <PremiumCard
      variant={variant === 'premium' ? 'highlight' : 'default'}
      glowColor={rankTheme.glow}
      className={cn('w-full max-w-[400px] hover:-translate-y-1 transition-transform', className)}
    >
      {/* Top Rank Badge (Floating) */}
      {rank <= 3 && (
        <div className="absolute -top-3 left-1/2 transform -translate-x-1/2 z-20">
          <div className="bg-gradient-to-r from-blue-600 to-cyan-500 text-white text-[10px] font-bold px-3 py-0.5 rounded-full shadow-lg flex items-center gap-1 uppercase tracking-wider">
            {rankTheme.label}
          </div>
        </div>
      )}

      <div className="p-6 pt-10 flex flex-col h-full relative z-10">

        {/* Header Section */}
        <div className="text-center mb-6">
          <h3 className="text-sm font-bold text-gray-400 uppercase tracking-widest mb-2">
            {rank > 3 ? rankTheme.label : 'Stock Symbol'}
          </h3>

          <div className="flex items-center justify-center gap-2 mb-1">
            <span className={cn("text-5xl font-black tracking-tighter", rankTheme.color)}>
              {symbol}
            </span>
          </div>

          <div className="flex items-center justify-center gap-2 text-gray-400 text-sm font-medium">
            <span>{formatCurrency(price, currency)}</span>
          </div>
        </div>

        {/* Metrics List */}
        <div className="space-y-3 mb-6 flex-grow">
          {/* EPS Growth */}
          <div className="flex items-center justify-between group/feature p-2 rounded-lg hover:bg-white/5 transition-colors">
            <div className="flex items-center gap-3">
              <div className={cn("p-1.5 rounded-md", isPositiveGrowth ? "bg-green-500/20 text-green-400" : "bg-red-500/20 text-red-400")}>
                <TrendingUp className="w-4 h-4" />
              </div>
              <span className="text-sm text-gray-300 font-medium">EPS Growth</span>
            </div>
            <span className={cn("font-bold text-sm", isPositiveGrowth ? "text-green-400" : "text-red-400")}>
              {formatPercentage(epsGrowth)}
            </span>
          </div>

          {/* Next Action & Progress Bar */}
          <div className="flex flex-col gap-2 p-3 rounded-xl bg-white/5 group/feature hover:bg-white/10 transition-colors">
            <div className="flex items-center justify-between mb-1">
              <div className="flex items-center gap-2">
                <div className="p-1.5 rounded-md bg-blue-500/20 text-blue-400">
                  <Calendar className="w-3.5 h-3.5" />
                </div>
                <span className="text-xs text-gray-400 font-medium uppercase tracking-wider">Next Action</span>
              </div>
              <span className="font-bold text-sm text-white">
                {daysUntilNextAction !== undefined ? `${daysUntilNextAction} Days` : 'N/A'}
              </span>
            </div>

            {/* Progress Bar */}
            <div className="h-1.5 w-full bg-gray-700/50 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-blue-500 to-cyan-400 rounded-full relative"
                style={{
                  width: `${daysUntilNextAction !== undefined ? Math.max(5, Math.min(100, ((90 - daysUntilNextAction) / 90) * 100)) : 0}%`
                }}
              >
                <div className="absolute inset-0 bg-white/20" />
              </div>
            </div>
          </div>
        </div>

        {/* Action Button */}
        <div className="mt-auto">
          <a
            href={`https://www.tradingview.com/symbols/${symbol}`}
            target="_blank"
            rel="noopener noreferrer"
            className="block w-full"
          >
            <button
              className="w-full py-3 rounded-xl font-bold text-sm transition-all duration-300 relative overflow-hidden bg-gradient-to-r from-blue-600 to-blue-700 text-white hover:shadow-lg hover:shadow-blue-500/25 group"
            >
              <span className="relative flex items-center justify-center gap-2">
                View Details
                <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
              </span>
            </button>
          </a>
        </div>

      </div>

      {/* Background Decor */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-blue-500/10 rounded-full blur-3xl -z-10 translate-x-10 -translate-y-10" />
      <div className="absolute bottom-0 left-0 w-24 h-24 bg-purple-500/10 rounded-full blur-3xl -z-10 -translate-x-10 translate-y-10" />

    </PremiumCard>
  );
};

StockDataCard.displayName = 'StockDataCard';

export const StockDataCardSkeleton = ({ className }: { className?: string }) => {
  return (
    <div className={cn("relative rounded-2xl border border-white/5 bg-gray-900/60 p-6 flex flex-col h-[350px] animate-pulse", className)}>
      <div className="h-4 w-24 bg-gray-800 rounded mx-auto mb-6" />
      <div className="h-12 w-32 bg-gray-800 rounded mx-auto mb-2" />
      <div className="h-4 w-20 bg-gray-800 rounded mx-auto mb-8" />

      <div className="space-y-4 mb-8">
        <div className="h-8 bg-gray-800 rounded w-full" />
        <div className="h-8 bg-gray-800 rounded w-full" />
      </div>

      <div className="mt-auto h-12 bg-gray-800 rounded-xl w-full" />
    </div>
  )
}

StockDataCardSkeleton.displayName = 'StockDataCardSkeleton';

export default StockDataCard;
