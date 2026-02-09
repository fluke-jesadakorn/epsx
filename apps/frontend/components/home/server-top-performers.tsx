import { getRankingsAction } from '@/app/actions/analytics';
import { StockDataCard } from '@/shared/components';

interface QuarterlyPerformance {
  quarter: string;
  date: string;
  price: number;
  eps: number;
  eps_growth: number;
  price_growth: number;
}

interface NextQuarterEstimate {
  quarter: string;
  estimated_eps: number;
  announcement_date: string;
  announcement_timestamp: number;
  days_until_announcement: number;
  confidence: string;
}

interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: QuarterlyPerformance[];
  next_quarter_estimate?: NextQuarterEstimate;
  currency: string;
}

interface ServerTopPerformersProps {
  className?: string;
}

const TopPerformersBox = ({ top3Data }: { top3Data: SymbolCardData[] }) => {
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
      <div className="grid grid-cols-1 justify-items-center gap-6 px-4 sm:grid-cols-2 lg:grid-cols-3">
        {top3Data.map(cardData => {
          const latestQuarter = cardData.quarterly_performance[0];

          return (
            <StockDataCard
              key={cardData.symbol}
              symbol={cardData.symbol}
              rank={cardData.rank}
              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              epsGrowth={latestQuarter?.eps_growth || 0}
              // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              price={latestQuarter?.price || 0}
              currency={cardData.currency}
              daysUntilNextAction={cardData.next_quarter_estimate?.days_until_announcement}
              // Always use premium style for top performers on homepage
              variant="premium"
            />
          );
        })}
      </div>
    </div>
  );
};

export default async function ServerTopPerformers({ className }: ServerTopPerformersProps) {
  let data: SymbolCardData[] = [];
  let error: string | null = null;

  try {
    const result = await getRankingsAction({
      page: 1,
      limit: 3,
      sort_by: 'growth_factor'
    } as any);

    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (result.data && Array.isArray(result.data)) {
      data = (result.data as any[]).map((ranking: any, index: number) => {
        const qData = ranking?.quarterly_performance ?? ranking?.quarterly_data ?? [];

        return {
          rank: ranking?.rank ?? ranking?.ranking_position ?? index + 1,
          symbol: ranking?.symbol ?? '',
          latest_date: qData[0]?.date ?? ranking?.latest_date ?? new Date().toISOString(),
          value: ranking?.value ?? ranking?.price_current ?? 0,
          active_status: ranking?.active_status ?? 'unknown',
          quarterly_performance: qData.map((q: any) => ({
            quarter: q?.quarter ?? '',
            date: q?.date ?? '',
            price: q?.price ?? 0,
            eps: q?.eps ?? 0,
            eps_growth: q?.eps_growth ?? 0,
            price_growth: q?.price_growth ?? 0,
          })),
          next_quarter_estimate: ranking?.next_quarter_estimate ? {
            quarter: ranking.next_quarter_estimate.quarter ?? '',
            estimated_eps: ranking.next_quarter_estimate.estimated_eps ?? 0,
            announcement_date: ranking.next_quarter_estimate.announcement_date ?? '',
            announcement_timestamp: ranking.next_quarter_estimate.announcement_timestamp ?? 0,
            days_until_announcement: ranking.next_quarter_estimate.days_until_announcement ?? 0,
            confidence: ranking.next_quarter_estimate.confidence ?? 'Medium'
          } : undefined,
          currency: 'USD'
        };
      });
    } else {
      error = (result as any).message ?? 'No valid data received';
    }
  } catch (err) {
      // Error logged silently
    error = err instanceof Error ? err.message : 'Unknown error';
  }

  return (
    <div className={`container mx-auto px-4 py-12 ${className ?? ''}`}>
      <div className="relative">
        <div className="absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/10 dark:to-yellow-600/10 blur-xl" />
        <div className="absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 dark:from-blue-700/10 dark:to-cyan-700/10 blur-xl" />

        {error ? (
          <div className="flex w-full flex-col gap-8">
            <div className="mb-6 space-y-4 text-center">
              <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
                Top Performing Companies
              </h2>
              <p className="text-muted-foreground mx-auto max-w-2xl">
                Discover the data leaders with exceptional growth and performance metrics
              </p>
              <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
              <p className="mt-8 text-gray-600 dark:text-gray-400">
                Unable to load data at this time. Please try again later.
              </p>
            </div>
          </div>
        ) : data.length === 0 ? (
          <div className="flex w-full flex-col gap-8">
            <div className="mb-6 space-y-4 text-center">
              <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
                Top Performing Companies
              </h2>
              <p className="text-muted-foreground mx-auto max-w-2xl">
                Discover the data leaders with exceptional growth and performance metrics
              </p>
              <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
              <p className="mt-8 text-gray-600 dark:text-gray-400">
                No data available at this time.
              </p>
            </div>
          </div>
        ) : (
          <TopPerformersBox top3Data={data} />
        )}
      </div>
    </div>
  );
}