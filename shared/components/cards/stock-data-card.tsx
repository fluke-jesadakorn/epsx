
import { ArrowRight, Calendar, Heart, TrendingUp } from 'lucide-react';
import { cn } from '../../utils';
import { PremiumCard } from '../ui/premium-card';

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
  companyName?: string;
  variant?: 'premium' | 'standard';
  className?: string;
  isWatchlisted?: boolean;
  onWatchlistToggle?: (symbol: string) => void;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

const formatPercentage = (value: number): string => {
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

const formatCurrency = (value: number, currency = 'USD'): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
};

const getRankTheme = (rank: number): { color: string; glow: 'blue' | 'purple' | 'orange' | 'green' | 'none', label: string } => {
  if (rank === 1) { return { color: 'text-yellow-400', glow: 'orange', label: 'CHAMPION' }; }
  if (rank === 2) { return { color: 'text-slate-300', glow: 'blue', label: 'ELITE' }; }
  if (rank === 3) { return { color: 'text-amber-500', glow: 'orange', label: 'LEGEND' }; }
  if (rank === 4) { return { color: 'text-emerald-400', glow: 'green', label: 'MASTER' }; }
  if (rank === 5) { return { color: 'text-teal-400', glow: 'green', label: 'EXPERT' }; }
  return { color: 'text-blue-500', glow: 'none', label: `RANK #${rank}` };
};

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function WatchlistButton({ symbol, isWatchlisted, onWatchlistToggle }: {
  symbol: string;
  isWatchlisted?: boolean;
  onWatchlistToggle: (symbol: string) => void;
}) {
  return (
    <button
      type="button"
      onClick={(e) => { e.preventDefault(); e.stopPropagation(); onWatchlistToggle(symbol); }}
      className="absolute top-3 right-3 z-20 p-1.5 rounded-full transition-colors hover:bg-black/[0.05] dark:hover:bg-white/10"
      aria-label={isWatchlisted === true ? 'Remove from watchlist' : 'Add to watchlist'}
    >
      <Heart className={cn('w-4 h-4 transition-colors', isWatchlisted === true ? 'fill-pink-500 text-pink-500' : 'text-gray-400 hover:text-pink-400')} />
    </button>
  );
}

function RankBadge({ rank, label }: { rank: number; label: string }) {
  return (
    <div className="absolute top-3 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-20">
      <div className={cn(
        "text-white text-[10px] font-bold px-4 py-1.5 rounded-full shadow-lg uppercase tracking-wider",
        rank <= 3 ? "bg-gradient-to-r from-blue-600 to-cyan-500" : "bg-gradient-to-r from-emerald-600 to-teal-500"
      )}>
        {label}
      </div>
    </div>
  );
}

function CardHeader({ rank, rankTheme, symbol, companyName, price, currency }: {
  rank: number;
  rankTheme: ReturnType<typeof getRankTheme>;
  symbol: string;
  companyName?: string;
  price: number;
  currency: string;
}) {
  return (
    <div className="text-center mb-4">
      <h3 className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-widest mb-1">
        {rank > 5 ? rankTheme.label : 'Stock Symbol'}
      </h3>
      <div className="flex items-center justify-center mb-0.5">
        <span className={cn("text-4xl font-black tracking-tighter", rankTheme.color)}>{symbol}</span>
      </div>
      {companyName !== undefined && companyName !== '' && (
        <div className="text-xs font-medium text-gray-500 dark:text-gray-400 truncate max-w-[90%] mx-auto">
          {companyName}
        </div>
      )}
      <div className="text-sm font-semibold text-gray-600 dark:text-gray-300 mt-0.5">
        {formatCurrency(price, currency)}
      </div>
    </div>
  );
}

function GrowthBar({ epsGrowth }: { epsGrowth: number }) {
  const isPositive = epsGrowth >= 0;
  const width = Math.max(5, Math.min(100, (Math.abs(epsGrowth) / 60) * 100));
  return (
    <div className="h-1.5 w-full bg-gray-200 dark:bg-gray-700/50 rounded-full overflow-hidden">
      <div
        className={cn('h-full rounded-full relative', isPositive ? 'bg-gradient-to-r from-green-500 to-emerald-400' : 'bg-gradient-to-r from-red-500 to-rose-400')}
        style={{ width: `${width}%` }}
      >
        <div className="absolute inset-0 bg-white/20" />
      </div>
    </div>
  );
}

function NextActionMetric({ daysUntilNextAction }: { daysUntilNextAction?: number }) {
  const progressWidth = daysUntilNextAction !== undefined
    ? Math.max(5, Math.min(100, ((90 - daysUntilNextAction) / 90) * 100))
    : 0;
  return (
    <div className="flex flex-col gap-2 p-3 rounded-xl bg-white/5 group/feature hover:bg-white/10 transition-colors">
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded-md bg-blue-500/20 text-blue-400">
            <Calendar className="w-3.5 h-3.5" />
          </div>
          <span className="text-xs text-gray-500 dark:text-gray-400 font-medium uppercase tracking-wider">Next Action</span>
        </div>
        <span className="font-bold text-sm text-gray-900 dark:text-white">
          {daysUntilNextAction !== undefined ? `${daysUntilNextAction} Days` : 'N/A'}
        </span>
      </div>
      <div className="h-1.5 w-full bg-gray-200 dark:bg-gray-700/50 rounded-full overflow-hidden">
        <div className="h-full bg-gradient-to-r from-blue-500 to-cyan-400 rounded-full relative" style={{ width: `${progressWidth}%` }}>
          <div className="absolute inset-0 bg-white/20" />
        </div>
      </div>
    </div>
  );
}

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
  companyName,
  variant = 'standard',
  className,
  isWatchlisted,
  onWatchlistToggle,
}: StockDataCardProps) => {
  const rankTheme = getRankTheme(rank);
  const isPositiveGrowth = epsGrowth >= 0;

  return (
    <PremiumCard
      variant='glass'
      glowColor={rankTheme.glow}
      className={cn('w-full hover:-translate-y-1 transition-transform', className)}
    >
      {onWatchlistToggle !== undefined && (
        <WatchlistButton symbol={symbol} isWatchlisted={isWatchlisted} onWatchlistToggle={onWatchlistToggle} />
      )}
      {rank <= 5 && <RankBadge rank={rank} label={rankTheme.label} />}

      <div className={cn("p-5 flex flex-col h-full relative z-10", rank <= 5 ? "pt-8" : "pt-5")}>
        <CardHeader rank={rank} rankTheme={rankTheme} symbol={symbol} companyName={companyName} price={price} currency={currency} />

        <div className="space-y-2 mb-4 flex-grow">
          <div className="flex flex-col gap-2 p-3 rounded-xl bg-white/5 hover:bg-white/10 transition-colors">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={cn("p-1.5 rounded-md", isPositiveGrowth ? "bg-green-500/20 text-green-400" : "bg-red-500/20 text-red-400")}>
                  <TrendingUp className="w-4 h-4" />
                </div>
                <span className="text-sm text-gray-600 dark:text-gray-300 font-medium">Growth</span>
              </div>
              <span className={cn("font-bold text-sm", isPositiveGrowth ? "text-green-400" : "text-red-400")}>
                {formatPercentage(epsGrowth)}
              </span>
            </div>
            <GrowthBar epsGrowth={epsGrowth} />
          </div>
          <NextActionMetric daysUntilNextAction={daysUntilNextAction} />
        </div>

        <div className="mt-auto">
          <a href={`https://www.tradingview.com/symbols/${symbol}`} target="_blank" rel="noopener noreferrer" className="block w-full">
            <button className="w-full py-3 rounded-xl font-bold text-sm transition-all duration-300 relative overflow-hidden bg-gradient-to-r from-blue-600 to-blue-700 text-white hover:shadow-lg hover:shadow-blue-500/25 group">
              <span className="relative flex items-center justify-center gap-2">
                View Details
                <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
              </span>
            </button>
          </a>
        </div>
      </div>

      <div className="absolute top-0 right-0 w-32 h-32 bg-blue-500/10 rounded-full blur-3xl -z-10 translate-x-10 -translate-y-10" />
      <div className="absolute bottom-0 left-0 w-24 h-24 bg-purple-500/10 rounded-full blur-3xl -z-10 -translate-x-10 translate-y-10" />
    </PremiumCard>
  );
};

StockDataCard.displayName = 'stock-data-card';

export const StockDataCardSkeleton = ({ className }: { className?: string }) => {
  return (
    <div className={cn("relative rounded-2xl border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-6 flex flex-col h-[350px] animate-pulse", className)}>
      <div className="h-4 w-24 bg-gray-200 dark:bg-gray-800 rounded mx-auto mb-6" />
      <div className="h-12 w-32 bg-gray-200 dark:bg-gray-800 rounded mx-auto mb-2" />
      <div className="h-4 w-20 bg-gray-200 dark:bg-gray-800 rounded mx-auto mb-8" />

      <div className="space-y-4 mb-8">
        <div className="h-8 bg-gray-200 dark:bg-gray-800 rounded w-full" />
        <div className="h-8 bg-gray-200 dark:bg-gray-800 rounded w-full" />
      </div>

      <div className="mt-auto h-12 bg-gray-200 dark:bg-gray-800 rounded-xl w-full" />
    </div>
  )
}

StockDataCardSkeleton.displayName = 'StockDataCardskeleton';

export default StockDataCard;
