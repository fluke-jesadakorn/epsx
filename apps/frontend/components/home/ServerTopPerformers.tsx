import { getAnalyticsData, type SymbolCardData } from '@/lib/unified-server-data';

interface ServerTopPerformersProps {
  className?: string;
}

const TopPerformersBox = ({ top3Data }: { top3Data: SymbolCardData[] }) => {
  const formatPercentage = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
  };

  const formatCurrency = (value: number, currency: string = 'USD') => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };

  return (
    <div className="flex w-full flex-col gap-8">
      <div className="mb-6 space-y-4 text-center">
        <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
          Top Performing Companies
        </h2>
        <p className="text-muted-foreground mx-auto max-w-2xl">
          Discover the data leaders with exceptional growth and performance metrics
        </p>
        <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
      </div>

      {/* Analytics-style card grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-6 sm:px-0">
        {top3Data.map(cardData => {
          const quarters = cardData.quarterly_performance?.slice(0, 2) || [];
          const latestQuarter = quarters[0];

          const getRankBadge = (rank: number) => {
            if (rank === 1) return { badge: '👑', color: 'from-yellow-400 to-orange-500' };
            if (rank === 2) return { badge: '🥈', color: 'from-slate-400 to-slate-500' };
            if (rank === 3) return { badge: '🥉', color: 'from-orange-400 to-amber-500' };
            return { badge: '🔥', color: 'from-green-400 to-emerald-500' };
          };

          const rankInfo = getRankBadge(cardData.rank);

          const getUltraPremiumStyle = (rank: number) => {
            if (rank === 1)
              return {
                container:
                  'bg-gradient-to-br from-yellow-50 via-amber-50 to-orange-50 border-4 border-yellow-400/50 shadow-yellow-500/40 hover:shadow-yellow-500/60 hover:border-yellow-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-yellow-300/30 to-transparent',
                glow: 'shadow-2xl shadow-yellow-500/50 hover:shadow-yellow-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-yellow-400/20 before:via-transparent before:to-yellow-400/20 before:blur-xl',
              };
            if (rank === 2)
              return {
                container:
                  'bg-gradient-to-br from-slate-50 via-gray-50 to-zinc-50 border-4 border-slate-400/50 shadow-slate-500/40 hover:shadow-slate-500/60 hover:border-slate-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-slate-300/30 to-transparent',
                glow: 'shadow-2xl shadow-slate-500/50 hover:shadow-slate-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-slate-400/20 before:via-transparent before:to-slate-400/20 before:blur-xl',
              };
            if (rank === 3)
              return {
                container:
                  'bg-gradient-to-br from-orange-50 via-amber-50 to-yellow-50 border-4 border-orange-400/50 shadow-orange-500/40 hover:shadow-orange-500/60 hover:border-orange-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-orange-300/30 to-transparent',
                glow: 'shadow-2xl shadow-orange-500/50 hover:shadow-orange-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-orange-400/20 before:via-transparent before:to-orange-400/20 before:blur-xl',
              };
            return {
              container:
                'bg-gradient-to-br from-white via-slate-50 to-gray-100 border-2 border-slate-300',
              shine: '',
              glow: '',
              halo: '',
            };
          };

          const ultraStyle = getUltraPremiumStyle(cardData.rank);

          return (
            <div
              key={cardData.symbol}
              className={`relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px] ${ultraStyle.container} dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 ${ultraStyle.glow} ${ultraStyle.halo}`}
            >
              {/* Animated shine effect */}
              <div
                className={`absolute inset-0 rounded-3xl ${ultraStyle.shine}`}
              ></div>

              {/* Ultra-premium floating rank badge */}
              <div
                className={`absolute -top-4 -left-4 h-16 w-16 bg-gradient-to-br ${rankInfo.color} flex rotate-12 transform items-center justify-center rounded-full border-4 border-white text-2xl text-white shadow-2xl z-30`}
              >
                {rankInfo.badge}
              </div>

              {/* Holographic corner effects */}
              <div
                className={`bg-gradient-radial absolute top-0 right-0 h-20 w-20 rounded-bl-3xl from-white/40 via-transparent to-transparent opacity-60`}
              ></div>
              <div
                className={`bg-gradient-radial absolute bottom-0 left-0 h-20 w-20 rounded-tr-3xl from-white/40 via-transparent to-transparent opacity-60`}
              ></div>

              {/* Premium content wrapper with extra padding */}
              <div className="p-8 pt-16">
                {/* Ultra-premium header with luxury typography */}
                <div className="mb-6 text-center">
                  <div className="mb-3">
                    <div className="mb-1 text-xs font-light tracking-[0.2em] text-slate-400 uppercase">
                      {cardData.rank === 1
                        ? '👑 CHAMPION'
                        : cardData.rank === 2
                          ? '🥈 ELITE'
                          : cardData.rank === 3
                            ? '🥉 LEGEND'
                            : '🔥 LEADER'}{' '}
                      · RANK #{cardData.rank}
                    </div>
                    <h3 className="mb-2 bg-gradient-to-r from-slate-700 via-slate-900 to-slate-700 dark:from-slate-200 dark:via-white dark:to-slate-200 bg-clip-text text-2xl font-black tracking-tight text-transparent sm:text-3xl">
                      {cardData.symbol}
                    </h3>
                    <div className="mx-auto h-px w-16 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>

                  <div className="flex justify-center">
                    <a
                      href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={`inline-flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-bold text-white shadow-xl ${
                        cardData.active_status === 'TRACK'
                          ? 'bg-gradient-to-r from-green-600 via-green-500 to-emerald-600'
                          : cardData.active_status === 'STOP'
                            ? 'bg-gradient-to-r from-red-600 via-red-500 to-rose-600'
                            : 'bg-gradient-to-r from-orange-600 via-orange-500 to-amber-600'
                      }`}
                    >
                      <span className="text-base sm:text-lg">📊</span>
                      <span className="tracking-wide text-sm sm:text-base">VIEW DETAILS</span>
                      <span className="text-xs opacity-75">→</span>
                    </a>
                  </div>
                </div>

                {/* Ultra-premium status section */}
                <div className="mb-6">
                  <div className="mb-4 flex items-center justify-between">
                    <div
                      className={`relative inline-flex items-center gap-3 rounded-2xl border-2 px-4 py-2 text-sm font-bold backdrop-blur-md ${
                        cardData.active_status === 'TRACK'
                          ? 'border-green-400/50 bg-green-500/20 text-green-900 shadow-green-400/20'
                          : cardData.active_status === 'STOP'
                            ? 'border-red-400/50 bg-red-500/20 text-red-900 shadow-red-400/20'
                            : 'border-orange-400/50 bg-orange-500/20 text-orange-900 shadow-orange-400/20'
                      } shadow-lg`}
                    >
                      <div
                        className={`h-3 w-3 rounded-full ${
                          cardData.active_status === 'TRACK'
                            ? 'bg-green-500 shadow-lg shadow-green-500/50'
                            : cardData.active_status === 'STOP'
                              ? 'bg-red-500 shadow-lg shadow-red-500/50'
                              : 'bg-orange-500 shadow-lg shadow-orange-500/50'
                        }`}
                      ></div>
                      <span className="tracking-wide">
                        {cardData.active_status === 'TRACK'
                          ? 'ACTIVE'
                          : cardData.active_status === 'STOP'
                            ? 'INACTIVE'
                            : cardData.active_status}
                      </span>
                    </div>

                    <div className="text-center">
                      <div className="mb-1 text-xs font-light tracking-wider text-slate-400 dark:text-slate-300 uppercase">
                        Next Report
                      </div>
                      <div
                        className={`text-lg font-bold ${
                          cardData.active_status === 'TRACK'
                            ? 'text-green-700 dark:text-green-400'
                            : cardData.active_status === 'STOP'
                              ? 'text-red-700 dark:text-red-400'
                              : 'text-orange-700 dark:text-orange-400'
                        }`}
                      >
                        {cardData.next_quarter_estimate?.days_until_announcement || 0}d
                      </div>
                    </div>
                  </div>

                  {/* Luxury progress indicator */}
                  <div className="relative h-2 overflow-hidden rounded-full bg-gradient-to-r from-slate-200 via-slate-100 to-slate-200 dark:from-slate-600 dark:via-slate-500 dark:to-slate-600">
                    <div className="absolute inset-0 bg-gradient-to-r from-white/50 dark:from-slate-300/30 via-transparent to-white/50 dark:to-slate-300/30"></div>
                    <div
                      className={`relative h-full overflow-hidden rounded-full ${
                        cardData.active_status === 'TRACK'
                          ? 'bg-gradient-to-r from-green-500 via-emerald-400 to-green-600'
                          : cardData.active_status === 'STOP'
                            ? 'bg-gradient-to-r from-red-500 via-rose-400 to-red-600'
                            : 'bg-gradient-to-r from-orange-500 via-amber-400 to-orange-600'
                      }`}
                      style={{
                        width: `${Math.max(0, Math.min(100, ((90 - (cardData.next_quarter_estimate?.days_until_announcement || 0)) / 90) * 100))}%`,
                      }}
                    >
                      <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"></div>
                    </div>
                  </div>
                </div>

                {/* Ultra-premium data showcase */}
                <div className="flex flex-col gap-4">
                  <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
                    <div className="mb-4 flex items-center justify-center gap-3">
                      <span
                        className={`text-xl sm:text-2xl`}
                      >{(latestQuarter?.eps_growth || 0) >= 0 ? '📈' : '📉'}</span>
                      <div className="text-center">
                        <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                          Growth
                        </div>
                      </div>
                    </div>
                    <div
                      className={`mb-2 text-center text-lg leading-tight font-black sm:text-2xl ${
                        (latestQuarter?.eps_growth || 0) >= 0
                          ? 'text-green-700 dark:text-green-400'
                          : 'text-red-700 dark:text-red-400'
                      }`}
                    >
                      {formatPercentage(latestQuarter?.eps_growth || 0)}
                    </div>
                    <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>

                  <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
                    <div className="mb-4 flex items-center justify-center gap-3">
                      <span className="text-xl sm:text-2xl">💰</span>
                      <div className="text-center">
                        <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                          Price
                        </div>
                      </div>
                    </div>
                    <div className="mb-2 text-center text-lg leading-tight font-black text-slate-800 dark:text-slate-200 sm:text-2xl">
                      {formatCurrency(latestQuarter?.price || 0, cardData.currency)}
                    </div>
                    <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default async function ServerTopPerformers({ className }: ServerTopPerformersProps) {
  try {
    // Fetch top 3 performing companies from backend
    const data = await getAnalyticsData({
      page: 1,
      limit: 3,
      sort_by: 'growth_factor',
    });

    if (!data || !data.rankings || data.rankings.length === 0) {
      return (
        <div className="container mx-auto px-4 py-12">
          <div className="text-center">
            <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
              Top Performing Companies
            </h2>
            <p className="text-muted-foreground mx-auto max-w-2xl mt-4">
              Discover the data leaders with exceptional growth and performance metrics
            </p>
            <div className="pancake-gradient mx-auto h-1 w-24 rounded-full mt-4" />
            <p className="mt-8 text-gray-600 dark:text-gray-400">
              Loading top performers...
            </p>
          </div>
        </div>
      );
    }

    // Get the top 3 performers
    const top3Data = data.rankings.slice(0, 3);

    return (
      <div className={`container mx-auto px-4 py-12 ${className || ''}`}>
        <div className="relative">
          <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
          <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 blur-xl" />
          
          <TopPerformersBox top3Data={top3Data} />
        </div>
      </div>
    );
  } catch (error) {
    console.error('Failed to fetch top performers:', error);
    
    return (
      <div className="container mx-auto px-4 py-12">
        <div className="text-center">
          <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
            Top Performing Companies
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl mt-4">
            Discover the data leaders with exceptional growth and performance metrics
          </p>
          <div className="pancake-gradient mx-auto h-1 w-24 rounded-full mt-4" />
          <p className="mt-8 text-gray-600 dark:text-gray-400">
            Unable to load data at this time. Please try again later.
          </p>
        </div>
      </div>
    );
  }
}